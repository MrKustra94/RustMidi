extern crate core;

mod extensions;
mod kubernetes;
mod midi;
mod scheduler;
mod worker;

use crate::kubernetes::model::deployment_id;
use crate::midi::controller::midir;
use crate::midi::model::{MidiMessage, MidiSender};
use crate::midi::registry::inmem::{K8sMidiMapping, ReadOnlyMidiRegistryConfig};
use crate::midi::registry::model::pad_mapping;
use crate::scheduler::Seconds;
use crate::worker::{WorkerK8sClient, WorkerMidiRegistry, WorkerMidiSender};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configs: Vec<K8sMidiMapping> = (0x00 as u8..0x7f as u8)
        .into_iter()
        .map(|fst_db| {
            let deployment = format!("test-{}", fst_db);
            K8sMidiMapping {
                deployment_id: deployment_id(deployment.clone(), deployment.clone(), deployment),
                pad_midi_mapping: pad_mapping(0x97, fst_db, 0x55, 0x70, 0x40, 0x20).unwrap(),
            }
        })
        .collect();

    // Schedule tasks first
    let (scheduler, queue) = scheduler::Scheduler::make();
    configs.iter().for_each(|k8s_m| {
        let _ = scheduler.register(scheduler::DeploymentCheckTask {
            deployment_id: k8s_m.deployment_id.clone(),
            schedule_every_seconds: Seconds(15),
        });
    });

    //set-up workers
    let config = ReadOnlyMidiRegistryConfig { mappings: configs };

    let client: WorkerK8sClient = Arc::new(kubernetes::stubs::AlwaysSuccess);
    let midi_registry: WorkerMidiRegistry =
        Arc::new(midi::registry::inmem::ReadOnlyMapMidiRegistry::new(config));
    let controller: WorkerMidiSender = Arc::new(midir::MidirBased::new("DDJ-XP2")?);
    let requests_queue = queue.0;

    let mut workers = Vec::with_capacity(4);
    for _ in 1..4 {
        let worker = worker::Worker::start_worker(
            client.clone(),
            midi_registry.clone(),
            controller.clone(),
            requests_queue.clone(),
        );
        workers.push(worker)
    }

    for worker in workers {
        let _ = worker.0.await;
    }

    Ok(())
}
