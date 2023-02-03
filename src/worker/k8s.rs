use crate::kubernetes::model::{DeploymentId, DeploymentStatus, K8sClient};
use crate::midi::model::DataByte;
use crate::midi::registry::model::MidiRegistry;
use crate::{MidiMessage, MidiSender};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;

pub type WorkerK8sClient = Arc<dyn K8sClient + Send + Sync + 'static>;
pub type WorkerMidiRegistry = Arc<dyn MidiRegistry + Send + Sync + 'static>;
pub type WorkerMidiSender = Arc<dyn MidiSender + Send + Sync + 'static>;

/// CheckDeploymentHandler is responsible for checking deployment status.
/// This component can be considered stateless - it only contains references to other services, which can be mocked.
/// It could be possible to share it across different components.
pub struct CheckDeploymentHandler {
    k8s_client: WorkerK8sClient,
    midi_registry: WorkerMidiRegistry,
    midi_sender: WorkerMidiSender,
}

impl CheckDeploymentHandler {
    pub fn new(
        k8s_client: WorkerK8sClient,
        midi_registry: WorkerMidiRegistry,
        midi_sender: WorkerMidiSender,
    ) -> CheckDeploymentHandler {
        CheckDeploymentHandler {
            k8s_client: k8s_client.clone(),
            midi_registry: midi_registry.clone(),
            midi_sender: midi_sender.clone(),
        }
    }

    async fn check(&self, request: &DeploymentId) {
        if let Some(mapping) = self.midi_registry.get(request) {
            let msg_constructor = |data_byte: DataByte| MidiMessage {
                status: mapping.status,
                fst_data_byte: mapping.fst_data_byte,
                snd_data_byte: data_byte,
            };

            let _ = self
                .midi_sender
                .send(msg_constructor(mapping.yellow_data_byte));

            let deployment_status = self
                .k8s_client
                .check_deployment(&request.context, &request.namespace, &request.deployment)
                .await;

            let msg = match deployment_status {
                Ok(DeploymentStatus::OK) => msg_constructor(mapping.green_data_byte),
                Ok(DeploymentStatus::NonOK) => msg_constructor(mapping.red_data_byte),
                Err(_) => msg_constructor(mapping.orange_data_byte),
            };
            let _ = self.midi_sender.send(msg);
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Seconds(pub u64);

#[derive(Clone)]
pub struct K8sWorkerContext {
    pub deployment_id: DeploymentId,
    pub schedule_every_seconds: Seconds,
}

/// K8sWorker contains the handle which encapsulates background task.
pub struct K8sWorker(pub JoinHandle<()>);

impl K8sWorker {
    pub fn start_worker(
        handler: Arc<CheckDeploymentHandler>,
        deployment: &K8sWorkerContext,
    ) -> K8sWorker {
        let shared_deployment = Arc::new(deployment.clone());
        K8sWorker(spawn(async move {
            loop {
                handler.check(&shared_deployment.deployment_id).await;
                tokio::time::sleep(Duration::from_secs(
                    shared_deployment.schedule_every_seconds.0,
                ))
                .await;
            }
        }))
    }
}
