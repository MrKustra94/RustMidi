use std::collections::HashMap;

use crate::worker::actor::{PadHandler, PadOutput};
use async_process;

pub struct Config {
    pub envs: HashMap<String, String>,
    pub command: String,
    pub args: Vec<String>,
}

pub struct ScriptHandler {
    config: Config,
}

impl ScriptHandler {
    pub fn new(config: Config) -> ScriptHandler {
        ScriptHandler { config }
    }
}

#[async_trait::async_trait]
impl PadHandler for ScriptHandler {
    async fn handle(&mut self) -> PadOutput {
        let command = async_process::Command::new(&self.config.command)
            .args(&self.config.args)
            .envs(&self.config.envs)
            .output()
            .await;

        match command {
            Ok(status) => {
                if status.status.success() {
                    PadOutput::Ok
                } else {
                    PadOutput::NotOk
                }
            }
            Err(_) => PadOutput::TempError,
        }
    }
}
