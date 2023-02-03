pub mod k8s;
pub mod script;

use crate::kubernetes::model::K8sClient;
use crate::MidiSender;
use serde::Deserialize;
use std::sync::Arc;

pub type WorkerK8sClient = Arc<dyn K8sClient + Send + Sync + 'static>;
pub type WorkerMidiSender = Arc<dyn MidiSender + Send + Sync + 'static>;

#[derive(Clone, Debug, Deserialize)]
pub struct Seconds(pub u64);
