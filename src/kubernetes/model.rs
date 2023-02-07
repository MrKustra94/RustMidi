use async_trait::async_trait;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub struct ClusterContext(pub String);

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub struct Namespace(pub String);

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub struct DeploymentName(pub String);

#[derive(Clone, Debug, Deserialize, Hash, Eq, PartialEq)]
pub struct DeploymentId {
    pub context: ClusterContext,
    pub namespace: Namespace,
    pub deployment: DeploymentName,
}

#[derive(Debug)]
pub enum DeploymentStatus {
    OK,
    InProgress,
    NonOK,
    Unknown,
}

#[async_trait]
pub trait K8sClient {
    async fn check_deployment(
        &self,
        deployment_id: &DeploymentId,
    ) -> anyhow::Result<DeploymentStatus>;
}
