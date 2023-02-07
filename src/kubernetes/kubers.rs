use async_trait::async_trait;

use k8s_openapi::api::apps::v1::{Deployment, DeploymentCondition};
use kube_client::config::KubeConfigOptions;
use kube_client::{Api, Client, Config};

use crate::kubernetes::model::DeploymentStatus::{InProgress, NonOK, Unknown, OK};
use crate::kubernetes::model::{DeploymentId, DeploymentStatus, K8sClient};

pub struct KubeRsBased;

const AVAILABLE: &str = "Available";
const PROGRESSING: &str = "Progressing";
const REPLICA_FAILURE: &str = "ReplicaFailure";

const TRUE_COND_STATUS: &str = "True";
const FALSE_COND_STATUS: &str = "False";

fn status_to_bool(status: &str) -> Option<bool> {
    match status {
        TRUE_COND_STATUS => Some(true),
        FALSE_COND_STATUS => Some(false),
        _ => None,
    }
}

#[derive(Default)]
struct DeploymentConditionsSummary {
    available: Option<bool>,
    progressing: Option<bool>,
    replica_failure: Option<bool>,
}

impl DeploymentConditionsSummary {
    fn merge(&mut self, condition: &DeploymentCondition) {
        let translated_status = status_to_bool(condition.status.as_str());
        match condition.type_.as_str() {
            AVAILABLE => self.available = translated_status,
            PROGRESSING => self.progressing = translated_status,
            REPLICA_FAILURE => self.replica_failure = translated_status,
            _ => (),
        }
    }

    fn status(&self) -> DeploymentStatus {
        match (self.available, self.progressing, self.replica_failure) {
            (_, _, Some(true)) => NonOK, // Failure happened. Indicate it immediately.
            (Some(true), Some(true), _) => OK, // Deployment is available. Indicate it immediately.
            (Some(true), Some(false), _) => NonOK, // Deployment old replicas are available, but deployment failed.
            (_, Some(true), _) => InProgress, // Deployment may not be available, but it is progressing.
            _ => Unknown,
        }
    }
}

#[async_trait]
impl K8sClient for KubeRsBased {
    async fn check_deployment(
        &self,
        deployment_id: &DeploymentId,
    ) -> anyhow::Result<DeploymentStatus> {
        let context_options = KubeConfigOptions {
            context: Some(deployment_id.context.0.clone()),
            ..Default::default()
        };
        let config = Config::from_kubeconfig(&context_options).await?;
        let client = Client::try_from(config)?;

        let deployment: Deployment = Api::namespaced(client, deployment_id.namespace.0.as_str())
            .get(deployment_id.deployment.0.as_str())
            .await?;

        let mut deployment_state: DeploymentConditionsSummary = Default::default();
        if let Some(conds) = deployment.status.and_then(|status| status.conditions) {
            conds
                .into_iter()
                .for_each(|cond| deployment_state.merge(&cond))
        }
        Ok(deployment_state.status())
    }
}
