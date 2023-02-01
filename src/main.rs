extern crate core;

mod configuration;
mod extensions;
mod kubernetes;
mod midi;
mod scheduler;
mod worker;

use crate::midi::model::{MidiMessage, MidiSender};
use crate::midi::registry::inmem::{K8sToPadMidiMapping, ReadOnlyMidiRegistryConfig};
use crate::scheduler::Seconds;
use crate::worker::{WorkerK8sClient, WorkerMidiRegistry, WorkerMidiSender};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = configuration::load_from_yaml("my_file.yaml")?;

    // Schedule tasks first
    let (scheduler, queue) = scheduler::Scheduler::make();
    for task in configuration::to_deployment_check_tasks(&configuration) {
        let _ = scheduler.register(task);
    }

    //set-up workers
    let registry_config = configuration::to_midi_registry_config(&configuration);
    let midi_registry: WorkerMidiRegistry = Arc::new(
        midi::registry::inmem::ReadOnlyMapMidiRegistry::new(registry_config),
    );
    let client: WorkerK8sClient = Arc::new(kubernetes::kubers::KubeRsBased);
    let sender: WorkerMidiSender = Arc::new(midi::controller::midir::MidirBased::new(
        configuration.controller_name.0.as_str(),
    )?);
    let requests_queue = queue.0;

    let worker = worker::Worker::start_worker(
        client.clone(),
        midi_registry.clone(),
        sender.clone(),
        requests_queue.clone(),
    );

    let _ = worker.0.await;

    Ok(())
}
