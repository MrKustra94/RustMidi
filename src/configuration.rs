use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::kubernetes::model as k8s_model;
use crate::midi::model as midi_model;
use crate::worker::actor;
use crate::worker::k8s as k8s_handler;
use crate::worker::script as script_handler;

// YAML specific configuration

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum HandlerConfig {
    K8S {
        #[serde(flatten)]
        deployment_id: k8s_model::DeploymentId,
        depl_in_progress: midi_model::DataByte,
        unknown: midi_model::DataByte,
    },
    Script {
        #[serde(default = "HashMap::new")]
        envs: HashMap<String, String>,
        command: String,
        #[serde(default = "Vec::new")]
        args: Vec<String>,
    },
}

#[derive(Debug, serde::Deserialize)]
pub struct PadConfig {
    #[serde(flatten)]
    pub pad_id: actor::PadId,
    pub every_seconds: u8,
    #[serde(flatten)]
    pub handler: HandlerConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct K8sMidiMapping {
    pub controller_name: Option<String>,
    pub color_palette: Arc<actor::ColorMapping>,
    pub mappings: Vec<PadConfig>,
}

// Parsed part - from configuration to application specific

pub enum ParsedHandlerConfig {
    K8S(k8s_handler::Config),
    Script(script_handler::Config),
}

impl From<HandlerConfig> for ParsedHandlerConfig {
    fn from(value: HandlerConfig) -> Self {
        match value {
            HandlerConfig::K8S {
                deployment_id,
                depl_in_progress,
                unknown,
            } => ParsedHandlerConfig::K8S(k8s_handler::Config {
                deployment_id,
                unknown,
                depl_in_progress,
            }),
            HandlerConfig::Script {
                envs,
                command,
                args,
            } => ParsedHandlerConfig::Script(script_handler::Config {
                envs,
                command,
                args,
            }),
        }
    }
}

pub struct ParsedPadConfig {
    pub actor_config: actor::Config,
    pub handler_config: ParsedHandlerConfig,
}

pub struct ParsedPadConfigs {
    pub controller_name: Option<String>,
    pub pad_configs: Vec<ParsedPadConfig>,
}

pub fn parse(midi_mapping: K8sMidiMapping) -> ParsedPadConfigs {
    let pad_configs: Vec<ParsedPadConfig> = midi_mapping
        .mappings
        .into_iter()
        .map(|config| ParsedPadConfig {
            actor_config: actor::Config {
                pad_mapping: actor::PadMapping {
                    pad_id: config.pad_id,
                    color_mapping: midi_mapping.color_palette.clone(),
                },
                schedule_every: Duration::from_secs(config.every_seconds.into()),
            },
            handler_config: config.handler.into(),
        })
        .collect();

    ParsedPadConfigs {
        controller_name: midi_mapping.controller_name,
        pad_configs,
    }
}

pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<K8sMidiMapping> {
    let conf_file = std::fs::File::open(path)?;
    // Workaround for merge anchors.
    // Useful for better file readability.
    // https://github.com/dtolnay/serde-yaml/issues/317
    let mut yaml_value: serde_yaml::Value = serde_yaml::from_reader(conf_file)?;
    yaml_value.apply_merge()?;
    Ok(serde_yaml::from_value(yaml_value)?)
}
