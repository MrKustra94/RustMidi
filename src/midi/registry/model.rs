use std::borrow::Cow;

use crate::kubernetes::model::DeploymentId;
use crate::midi::model::{DataByte, Status};

#[derive(Clone)]
pub struct PadMapping {
    pub status: Status,
    pub fst_data_byte: DataByte,
    pub green_data_byte: DataByte,
    pub yellow_data_byte: DataByte,
    pub orange_data_byte: DataByte,
    pub red_data_byte: DataByte,
}

pub trait MidiRegistry {
    fn get<'a, 's: 'a>(&'s self, deployment_id: &'a DeploymentId) -> Option<Cow<'a, PadMapping>>;
}
