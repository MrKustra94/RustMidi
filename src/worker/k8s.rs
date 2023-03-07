use crate::kubernetes::model::{DeploymentId, DeploymentStatus, K8sClient};
use crate::midi::model::DataByte;
use crate::worker::actor::{PadHandler, PadOutput};
use std::sync::Arc;

pub struct Config {
    pub deployment_id: DeploymentId,
    pub unknown: DataByte,
    pub depl_in_progress: DataByte,
}

pub struct K8SDeploymentHandler {
    k8s_client: Arc<dyn K8sClient + Send + Sync>,
    config: Config,
}

impl K8SDeploymentHandler {
    pub fn new(
        k8s_client: Arc<dyn K8sClient + Send + Sync>,
        config: Config,
    ) -> K8SDeploymentHandler {
        K8SDeploymentHandler { k8s_client, config }
    }
}

#[async_trait::async_trait]
impl PadHandler for K8SDeploymentHandler {
    async fn handle(&self) -> PadOutput {
        let deployment_id = &self.config.deployment_id;
        let deployment_status = self.k8s_client.check_deployment(deployment_id).await;

        match deployment_status {
            Ok(DeploymentStatus::OK) => PadOutput::Ok,
            Ok(DeploymentStatus::NonOK) => PadOutput::NotOk,
            Ok(DeploymentStatus::InProgress) => PadOutput::Custom(self.config.depl_in_progress),
            Ok(DeploymentStatus::Unknown) => PadOutput::Custom(self.config.unknown),
            Err(_) => PadOutput::TempError,
        }
    }
}
