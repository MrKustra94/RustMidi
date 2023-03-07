#[derive(Debug, serde::Deserialize)]
pub struct ClusterContext(pub String);

#[derive(Debug, serde::Deserialize)]
pub struct Namespace(pub String);

#[derive(Debug, serde::Deserialize)]
pub struct DeploymentName(pub String);

#[derive(Debug, serde::Deserialize)]
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

#[async_trait::async_trait]
pub trait K8sClient {
    async fn check_deployment(
        &self,
        deployment_id: &DeploymentId,
    ) -> anyhow::Result<DeploymentStatus>;
}
