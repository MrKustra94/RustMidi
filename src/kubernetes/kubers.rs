use async_trait::async_trait;

use k8s_openapi::api::apps::v1::Deployment;
use kube_client::config::KubeConfigOptions;
use kube_client::{Api, Client, Config};

use crate::kubernetes::model::DeploymentStatus::{NonOK, OK};
use crate::kubernetes::model::{
    ClusterContext, DeploymentName, DeploymentStatus, K8sClient, Namespace,
};

pub struct KubeRsBased;

#[async_trait]
impl K8sClient for KubeRsBased {
    async fn check_deployment(
        &self,
        context: &ClusterContext,
        namespace: &Namespace,
        deployment: &DeploymentName,
    ) -> anyhow::Result<DeploymentStatus> {
        let context_options = KubeConfigOptions {
            context: Some(context.0.clone()),
            ..Default::default()
        };
        let config = Config::from_kubeconfig(&context_options).await?;
        let client = Client::try_from(config)?;

        let deployment: Deployment = Api::namespaced(client, namespace.0.as_str())
            .get(deployment.0.as_str())
            .await?;
        if let Some(status) = deployment.status {
            if status.replicas == status.ready_replicas {
                Ok(OK)
            } else {
                Ok(NonOK)
            }
        } else {
            Ok(NonOK)
        }
    }
}
