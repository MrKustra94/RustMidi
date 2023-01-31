use async_trait::async_trait;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct ClusterContext(String);

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Namespace(String);

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct DeploymentName(String);

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct DeploymentId {
    pub context: ClusterContext,
    pub namespace: Namespace,
    pub deployment: DeploymentName,
}

pub fn deployment_id(context: String, namespace: String, deployment: String) -> DeploymentId {
    DeploymentId {
        context: ClusterContext(context),
        namespace: Namespace(namespace),
        deployment: DeploymentName(deployment),
    }
}

pub enum DeploymentStatus {
    OK,
    NonOK,
}

#[async_trait]
pub trait K8sClient {
    async fn check_deployment(
        &self,
        context: &ClusterContext,
        namespace: &Namespace,
        deployment: &DeploymentName,
    ) -> anyhow::Result<DeploymentStatus>;
}
