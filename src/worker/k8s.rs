use crate::kubernetes::model::{DeploymentId, DeploymentStatus};

use crate::midi::model::PadMapping;
use crate::worker::{Seconds, WorkerK8sClient, WorkerMidiSender};
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;

/// CheckDeploymentHandler is responsible for checking deployment status.
/// This component can be considered stateless - it only contains references to other services, which can be mocked.
/// It could be possible to share it across different components.
pub struct CheckDeploymentHandler {
    k8s_client: WorkerK8sClient,
    midi_sender: WorkerMidiSender,
}

impl CheckDeploymentHandler {
    pub fn new(
        k8s_client: WorkerK8sClient,
        midi_sender: WorkerMidiSender,
    ) -> CheckDeploymentHandler {
        CheckDeploymentHandler {
            k8s_client: k8s_client.clone(),
            midi_sender: midi_sender.clone(),
        }
    }

    async fn check(&self, context: &K8sContext) {
        let mapping = &context.pad_mapping;
        self.midi_sender.send_and_forget(mapping.yellow_message());

        let deployment_id = &context.deployment_id;
        let deployment_status = self.k8s_client.check_deployment(deployment_id).await;

        let msg = match deployment_status {
            Ok(DeploymentStatus::OK) => mapping.green_message(),
            Ok(DeploymentStatus::NonOK) => mapping.red_message(),
            Ok(DeploymentStatus::InProgress) => mapping.blue_message(),
            Ok(DeploymentStatus::Unknown) => mapping.white_message(),
            Err(_) => mapping.orange_message(),
        };
        self.midi_sender.send_and_forget(msg)
    }
}

#[derive(Clone)]
pub struct K8sContext {
    pub deployment_id: DeploymentId,
    pub schedule_every_seconds: Seconds,
    pub pad_mapping: PadMapping,
}

/// K8sWorker contains the handle which encapsulates background task.
pub struct K8sWorker(pub JoinHandle<()>);

impl K8sWorker {
    pub fn start_worker(handler: Arc<CheckDeploymentHandler>, context: K8sContext) -> K8sWorker {
        K8sWorker(spawn(async move {
            loop {
                handler.check(&context).await;
                tokio::time::sleep(Duration::from_secs(context.schedule_every_seconds.0)).await;
            }
        }))
    }
}
