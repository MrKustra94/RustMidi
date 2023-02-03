use crate::worker::{Seconds, WorkerMidiSender};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::midi::model::PadMapping;
use async_process;
use tokio::task::JoinHandle;

#[derive(Clone, Debug, Deserialize)]
pub struct Envs(pub HashMap<String, String>);

impl Envs {
    pub fn empty() -> Self {
        Envs(HashMap::new())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Command(String);

#[derive(Clone, Debug, Deserialize)]
pub struct CommandArgs(Vec<String>);

impl CommandArgs {
    pub fn empty() -> Self {
        CommandArgs(vec![])
    }
}

#[derive(Clone)]
pub struct ScriptContext {
    pub envs: Envs,
    pub command: Command,
    pub args: CommandArgs,
    pub pad_mapping: PadMapping,
    pub schedule_seconds: Seconds,
}

pub struct ScriptHandler {
    midi_sender: WorkerMidiSender,
}

impl ScriptHandler {
    pub fn new(midi_sender: WorkerMidiSender) -> ScriptHandler {
        ScriptHandler { midi_sender }
    }

    async fn handle(&self, context: &ScriptContext) {
        let mapping = &context.pad_mapping;

        self.midi_sender.send_and_forget(mapping.yellow_message());

        let command = async_process::Command::new(&context.command.0)
            .args(&context.args.0)
            .envs(context.envs.0.clone())
            .output()
            .await;

        match command {
            Ok(status) => {
                if status.status.success() {
                    self.midi_sender.send_and_forget(mapping.green_message());
                } else {
                    self.midi_sender.send_and_forget(mapping.red_message());
                }
            }
            Err(_) => {
                self.midi_sender.send_and_forget(mapping.orange_message());
            }
        }
    }
}

pub struct ScriptWorker(pub JoinHandle<()>);

impl ScriptWorker {
    pub fn start_worker(handler: Arc<ScriptHandler>, context: ScriptContext) -> ScriptWorker {
        ScriptWorker(tokio::spawn(async move {
            loop {
                handler.handle(&context).await;
                tokio::time::sleep(Duration::from_secs(context.schedule_seconds.0)).await;
            }
        }))
    }
}
