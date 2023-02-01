use std::borrow::Cow;
use std::collections::HashMap;

use crate::kubernetes::model::DeploymentId;
use crate::midi::registry::model::{MidiRegistry, PadMapping};

pub struct K8sToPadMidiMapping {
    pub deployment_id: DeploymentId,
    pub pad_midi_mapping: PadMapping,
}

pub struct ReadOnlyMidiRegistryConfig {
    pub mappings: Vec<K8sToPadMidiMapping>,
}

pub struct ReadOnlyMapMidiRegistry(HashMap<DeploymentId, Cow<'static, PadMapping>>);

impl ReadOnlyMapMidiRegistry {
    pub fn new(config: ReadOnlyMidiRegistryConfig) -> ReadOnlyMapMidiRegistry {
        ReadOnlyMapMidiRegistry(
            config
                .mappings
                .into_iter()
                .map(|mapping| (mapping.deployment_id, Cow::Owned(mapping.pad_midi_mapping)))
                .collect(),
        )
    }
}

impl MidiRegistry for ReadOnlyMapMidiRegistry {
    fn get<'a, 's: 'a>(&'s self, deployment_id: &'a DeploymentId) -> Option<Cow<'a, PadMapping>> {
        (&self.0).get(deployment_id).cloned()
    }
}
