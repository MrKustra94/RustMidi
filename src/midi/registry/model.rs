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

pub fn pad_mapping(
    status: u8,
    fst_data_byte: u8,
    green_data_byte: u8,
    yellow_data_byte: u8,
    orange_data_byte: u8,
    red_data_byte: u8,
) -> Option<PadMapping> {
    Status::from_u8(status)
        .zip(DataByte::from_u8(fst_data_byte))
        .zip(DataByte::from_u8(green_data_byte))
        .zip(DataByte::from_u8(yellow_data_byte))
        .zip(DataByte::from_u8(orange_data_byte))
        .zip(DataByte::from_u8(red_data_byte))
        .map(
            |(
                ((((status, fst_data_byte), green_data_byte), yellow_data_byte), orange_data_byte),
                red_data_byte,
            )| {
                PadMapping {
                    status,
                    fst_data_byte,
                    green_data_byte,
                    yellow_data_byte,
                    orange_data_byte,
                    red_data_byte,
                }
            },
        )
}

pub trait MidiRegistry {
    fn get<'a, 's: 'a>(&'s self, deployment_id: &'a DeploymentId) -> Option<Cow<'a, PadMapping>>;
}
