extern crate core;

use crate::actor::PadHandler;
use clap::Parser;
use std::sync::Arc;

use crate::configuration as conf;
use crate::midi::controller::midir;
use crate::midi::model as midi_model;
use crate::worker::{actor, k8s as k8s_handler, script as script_handler};

mod configuration;
mod extension;
mod kubernetes;
mod midi;
mod worker;

#[derive(clap::Parser)]
struct CLIArgs {
    #[arg(short = 'p', long, default_value = "midi_config.yaml")]
    pub config_path: String,
}

fn main() -> anyhow::Result<()> {
    let tokio_runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?,
    );

    let shared_runtime = tokio_runtime.clone();
    tokio_runtime.block_on(run_program(shared_runtime))
}

async fn run_program(tokio_runtime: Arc<tokio::runtime::Runtime>) -> anyhow::Result<()> {
    let cli_args = CLIArgs::parse();

    let parsed_config = load_and_parse(&cli_args.config_path)?;

    let k8s_client = Arc::new(kubernetes::kubers::KubeRsBased);
    let midi_sender: Arc<dyn midi_model::MidiSender + Send + Sync> = Arc::new(
        midir::MidirBasedSender::new(&parsed_config.controller_name)?,
    );
    let midi_receiver = midir::MidirBasedReceiver::new(&parsed_config.controller_name)?;

    let runtime = Arc::new(actor::TokioRuntime::new(tokio_runtime));
    let mut handles = Vec::new();
    let (handle, listener_actor) = actor::PadChangesListener::start(midi_receiver, runtime.clone());
    handles.push(handle);

    for pad_config in parsed_config.pad_configs {
        let handler: Arc<tokio::sync::Mutex<dyn PadHandler>> = match pad_config.handler_config {
            conf::ParsedHandlerConfig::K8S(config) => Arc::new(tokio::sync::Mutex::new(
                k8s_handler::K8SDeploymentHandler::new(k8s_client.clone(), config),
            )),
            conf::ParsedHandlerConfig::Script(config) => Arc::new(tokio::sync::Mutex::new(
                script_handler::ScriptHandler::new(config),
            )),
        };

        let (handle, actor) = actor::PadActor::start(
            handler,
            midi_sender.clone(),
            runtime.clone(),
            pad_config.actor_config,
        );

        listener_actor.register(actor.pad_id.clone(), Arc::new(actor));

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.0.await;
    }

    Ok(())
}

fn load_and_parse(config_path: &str) -> anyhow::Result<conf::ParsedPadConfigs> {
    let configuration = configuration::load_from_yaml(config_path)?;
    //set-up workers
    Ok(configuration::parse(configuration))
}
