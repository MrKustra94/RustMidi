use std::path::Path;

use serde::Deserialize;
use serde_yaml::Value;

use crate::kubernetes::model::{ClusterContext, DeploymentId, DeploymentName, Namespace};
use crate::midi::model::{ColorMapping, DataByte, PadMapping, Status};
use crate::worker::k8s;
use crate::worker::script;
use crate::worker::Seconds;

#[derive(Debug, Deserialize)]
pub struct PadColors {
    pub green: DataByte,
    pub red: DataByte,
    pub orange: DataByte,
    pub yellow: DataByte,
}

#[derive(Debug, Deserialize)]
pub struct K8sDeployment {
    pub context: ClusterContext,
    pub namespace: Namespace,
    pub deployment: DeploymentName,
}

#[derive(Debug, Deserialize)]
pub struct Pad {
    pub status: Status,
    pub fst_data_byte: DataByte,
}

#[derive(Debug, Deserialize)]
pub struct K8sPadMapping {
    #[serde(flatten)]
    pub deployment: DeploymentId,
    pub schedule_seconds: Seconds,
    #[serde(flatten)]
    pub pad: Pad,
}

#[derive(Debug, Deserialize)]
pub struct ScriptPadMapping {
    #[serde(default = "script::Envs::empty")]
    pub envs: script::Envs,
    pub command: script::Command,
    #[serde(default = "script::CommandArgs::empty")]
    pub args: script::CommandArgs,
    #[serde(flatten)]
    pub pad: Pad,
    pub schedule_seconds: Seconds,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigMapping {
    K8S(K8sPadMapping),
    Script(ScriptPadMapping),
}

#[derive(Debug, Deserialize)]
pub struct ControllerName(pub String);

#[derive(Debug, Deserialize)]
pub struct ControllerMappings {
    pub color_palette: PadColors,
    pub mappings: Vec<ConfigMapping>,
}

#[derive(Debug, Deserialize)]
pub struct K8sMidiMapping {
    pub controller_name: ControllerName,
    #[serde(flatten)]
    pub controller_mappings: ControllerMappings,
}

pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<K8sMidiMapping> {
    let conf_file = std::fs::File::open(path)?;
    // Workaround for merge anchors.
    // Useful for better file readability.
    // https://github.com/dtolnay/serde-yaml/issues/317
    let mut yaml_value: Value = serde_yaml::from_reader(conf_file)?;
    yaml_value.apply_merge()?;
    Ok(serde_yaml::from_value(yaml_value)?)
}

pub enum ParsedContext {
    K8S(k8s::K8sContext),
    Script(script::ScriptContext),
}

pub fn extract_contexts(controller_mappings: ControllerMappings) -> Vec<ParsedContext> {
    controller_mappings
        .mappings
        .into_iter()
        .map(|mapping| match mapping {
            ConfigMapping::K8S(k8s_m) => ParsedContext::K8S(k8s::K8sContext {
                deployment_id: k8s_m.deployment,
                schedule_every_seconds: k8s_m.schedule_seconds,
                pad_mapping: PadMapping {
                    status: k8s_m.pad.status,
                    fst_data_byte: k8s_m.pad.fst_data_byte,
                    color_mapping: ColorMapping {
                        green_data_byte: controller_mappings.color_palette.green,
                        yellow_data_byte: controller_mappings.color_palette.yellow,
                        orange_data_byte: controller_mappings.color_palette.orange,
                        red_data_byte: controller_mappings.color_palette.red,
                    },
                },
            }),
            ConfigMapping::Script(script_m) => ParsedContext::Script(script::ScriptContext {
                envs: script_m.envs,
                command: script_m.command,
                args: script_m.args,
                pad_mapping: PadMapping {
                    status: script_m.pad.status,
                    fst_data_byte: script_m.pad.fst_data_byte,
                    color_mapping: ColorMapping {
                        green_data_byte: controller_mappings.color_palette.green,
                        yellow_data_byte: controller_mappings.color_palette.yellow,
                        orange_data_byte: controller_mappings.color_palette.orange,
                        red_data_byte: controller_mappings.color_palette.red,
                    },
                },
                schedule_seconds: script_m.schedule_seconds,
            }),
        })
        .collect()
}
