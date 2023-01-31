use async_channel::Receiver;
use std::sync::Arc;
use tokio::spawn;
use tokio::task::JoinHandle;

use crate::kubernetes::model::{DeploymentId, DeploymentStatus, K8sClient};
use crate::midi::model::DataByte;
use crate::midi::registry::model::MidiRegistry;
use crate::{MidiMessage, MidiSender};

// Thread safe type aliases
pub type WorkerK8sClient = Arc<dyn K8sClient + Send + Sync + 'static>;
pub type WorkerMidiRegistry = Arc<dyn MidiRegistry + Send + Sync + 'static>;
pub type WorkerMidiSender = Arc<dyn MidiSender + Send + Sync + 'static>;

struct WorkerHandler {
    k8s_client: WorkerK8sClient,
    midi_registry: WorkerMidiRegistry,
    midi_sender: WorkerMidiSender,
}

impl WorkerHandler {
    async fn handle_request(&self, request: &DeploymentId) {
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

pub struct Worker(pub JoinHandle<()>);

impl Worker {
    pub fn start_worker(
        k8s_client: WorkerK8sClient,
        midi_registry: WorkerMidiRegistry,
        midi_sender: WorkerMidiSender,
        request_queue_r: Receiver<DeploymentId>,
    ) -> Worker {
        Worker(spawn(async move {
            let state = Arc::new(WorkerHandler {
                k8s_client,
                midi_registry,
                midi_sender,
            });
            while let Ok(request) = request_queue_r.recv().await {
                let local_state = state.clone();
                let _ = spawn(async move { local_state.clone().handle_request(&request).await });
            }
        }))
    }
}
