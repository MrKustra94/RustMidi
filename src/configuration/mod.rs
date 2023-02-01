use crate::kubernetes::model::{ClusterContext, DeploymentId, DeploymentName, Namespace};
use crate::midi::model::{DataByte, Status};
use crate::midi::registry::model::PadMapping;
use crate::scheduler::DeploymentCheckTask;
use crate::{K8sToPadMidiMapping, ReadOnlyMidiRegistryConfig, Seconds};
use serde::Deserialize;
use std::path::Path;

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
pub struct ControllerName(pub String);

#[derive(Debug, Deserialize)]
pub struct K8sMidiMapping {
    pub controller_name: ControllerName,
    pub color_palette: PadColors,
    pub mappings: Vec<K8sPadMapping>,
}

pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<K8sMidiMapping> {
    let conf_file = std::fs::File::open(path)?;
    Ok(serde_yaml::from_reader(conf_file)?)
}

pub fn to_deployment_check_tasks(mapping: &K8sMidiMapping) -> Vec<DeploymentCheckTask> {
    mapping
        .mappings
        .iter()
        .map(|mapping| DeploymentCheckTask {
            deployment_id: mapping.deployment.clone(),
            schedule_every_seconds: mapping.schedule_seconds.clone(),
        })
        .collect()
}

pub fn to_midi_registry_config(midi_mapping: &K8sMidiMapping) -> ReadOnlyMidiRegistryConfig {
    let config_mappings = midi_mapping
        .mappings
        .iter()
        .map(|mapping| K8sToPadMidiMapping {
            deployment_id: mapping.deployment.clone(),
            pad_midi_mapping: PadMapping {
                status: mapping.pad.status,
                fst_data_byte: mapping.pad.fst_data_byte,
                green_data_byte: midi_mapping.color_palette.green,
                yellow_data_byte: midi_mapping.color_palette.yellow,
                orange_data_byte: midi_mapping.color_palette.orange,
                red_data_byte: midi_mapping.color_palette.red,
            },
        });

    ReadOnlyMidiRegistryConfig {
        mappings: config_mappings.collect(),
    }
}
