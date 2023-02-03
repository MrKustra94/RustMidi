extern crate core;

mod configuration;
mod extensions;
mod kubernetes;
mod midi;
mod worker;

use crate::midi::controller::midir::MidirBased;
use crate::midi::model::{MidiMessage, MidiSender};
use crate::midi::registry::inmem::{K8sToPadMidiMapping, ReadOnlyMidiRegistryConfig};
use crate::worker::k8s::{
    CheckDeploymentHandler, K8sWorker, WorkerK8sClient, WorkerMidiRegistry, WorkerMidiSender,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = configuration::load_from_yaml("my_file.yaml")?;

    //set-up workers
    let registry_config = configuration::to_midi_registry_config(&configuration);
    let midi_registry: WorkerMidiRegistry = Arc::new(
        midi::registry::inmem::ReadOnlyMapMidiRegistry::new(registry_config),
    );

    let k8s_client: WorkerK8sClient = Arc::new(kubernetes::kubers::KubeRsBased);
    let midi_sender: WorkerMidiSender =
        Arc::new(MidirBased::new(configuration.controller_name.0.as_str())?);

    let deployment_check_handler = Arc::new(CheckDeploymentHandler::new(
        k8s_client,
        midi_registry,
        midi_sender,
    ));
    let k8s_workers_configs = configuration::to_k8s_worker_contexts(&configuration);
    let workers: Vec<K8sWorker> = k8s_workers_configs
        .into_iter()
        .map(|config| K8sWorker::start_worker(deployment_check_handler.clone(), &config))
        .collect();

    for worker in workers {
        let _ = worker.0.await;
    }

    Ok(())
}
