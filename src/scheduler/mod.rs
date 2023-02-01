use async_channel::{unbounded, Receiver, Sender};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::kubernetes::model::DeploymentId;

#[derive(Clone, Debug, Deserialize)]
pub struct Seconds(pub u64);

#[derive(Clone)]
pub struct DeploymentCheckTask {
    pub deployment_id: DeploymentId,
    pub schedule_every_seconds: Seconds,
}

type RequestQueue = Sender<DeploymentId>;

pub struct Scheduler {
    request_queues_senders: Arc<RequestQueue>,
    scheduled_tasks: Mutex<Vec<JoinHandle<()>>>,
}

pub struct RequestsQueue(pub Receiver<DeploymentId>);

impl Scheduler {
    pub fn register(&self, task: DeploymentCheckTask) -> anyhow::Result<()> {
        let sender = self.request_queues_senders.clone();

        if let Ok(mut tasks) = self.scheduled_tasks.lock() {
            let scheduling_handle = tokio::spawn(async move {
                loop {
                    let _ = sender.send(task.deployment_id.clone()).await;
                    tokio::time::sleep(Duration::from_secs(task.schedule_every_seconds.0)).await;
                }
            });
            tasks.push(scheduling_handle)
        }
        Ok(())
    }

    pub fn make() -> (Scheduler, RequestsQueue) {
        let (senders, receivers) = unbounded();
        let request_queues_senders = Arc::new(senders);
        let scheduled_tasks = Mutex::new(Vec::new());
        let scheduler = Scheduler {
            request_queues_senders,
            scheduled_tasks,
        };
        (scheduler, RequestsQueue(receivers))
    }
}
