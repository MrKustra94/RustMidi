use async_trait::async_trait;
use std::time::Duration;

use crate::kubernetes::model::{
    ClusterContext, DeploymentName, DeploymentStatus, K8sClient, Namespace,
};

pub struct AlwaysFail;

#[async_trait]
impl K8sClient for AlwaysFail {
    async fn check_deployment(
        &self,
        _context: &ClusterContext,
        _namespace: &Namespace,
        _deployment: &DeploymentName,
    ) -> anyhow::Result<DeploymentStatus> {
        Ok(DeploymentStatus::NonOK)
    }
}

pub struct AlwaysSuccess;

#[async_trait]
impl K8sClient for AlwaysSuccess {
    async fn check_deployment(
        &self,
        _context: &ClusterContext,
        _namespace: &Namespace,
        _deployment: &DeploymentName,
    ) -> anyhow::Result<DeploymentStatus> {
        tokio::time::sleep(Duration::from_secs(3)).await;
        Ok(DeploymentStatus::OK)
    }
}
