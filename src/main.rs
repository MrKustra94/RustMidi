extern crate core;

mod configuration;
mod extensions;
mod kubernetes;
mod midi;
mod worker;

use crate::configuration::ParsedContext;
use crate::midi::controller::midir::MidirBased;
use crate::midi::model::{MidiMessage, MidiSender};
use crate::worker::k8s::{CheckDeploymentHandler, K8sWorker};
use crate::worker::script::{ScriptHandler, ScriptWorker};
use crate::worker::{WorkerK8sClient, WorkerMidiSender};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = configuration::load_from_yaml("my_file.yaml")?;
    let controller_name = configuration.controller_name;
    let controller_mappings = configuration.controller_mappings;

    //set-up workers
    let contexts = configuration::extract_contexts(controller_mappings);

    let k8s_client: WorkerK8sClient = Arc::new(kubernetes::kubers::KubeRsBased);
    let midi_sender: WorkerMidiSender = Arc::new(MidirBased::new(controller_name.0.as_str())?);

    let check_deployment_handler = Arc::new(CheckDeploymentHandler::new(
        k8s_client.clone(),
        midi_sender.clone(),
    ));

    let script_handler = Arc::new(ScriptHandler::new(midi_sender.clone()));

    let mut k8s_workers = Vec::new();
    let mut script_workers = Vec::new();

    for context in contexts {
        match context {
            ParsedContext::K8S(k8s_ctx) => k8s_workers.push(K8sWorker::start_worker(
                check_deployment_handler.clone(),
                k8s_ctx,
            )),
            ParsedContext::Script(script_ctx) => script_workers.push(ScriptWorker::start_worker(
                script_handler.clone(),
                script_ctx,
            )),
        }
    }

    for worker in k8s_workers {
        let _ = worker.0.await;
    }

    for worker in script_workers {
        let _ = worker.0.await;
    }

    Ok(())
}
