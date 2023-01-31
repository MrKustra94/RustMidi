use crate::extensions::option::OptionExt;
use std::error;
use thiserror;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Status(u8);

const U8_MSB_EXTRACTOR: u8 = 0x80;

impl Status {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn from_u8(status: u8) -> Option<Status> {
        Option::when(status & U8_MSB_EXTRACTOR == U8_MSB_EXTRACTOR, || {
            Status(status)
        })
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DataByte(u8);

impl DataByte {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn from_u8(db: u8) -> Option<DataByte> {
        Option::when(db & U8_MSB_EXTRACTOR == 0, || DataByte(db))
    }
}

pub struct MidiMessage {
    pub status: Status,
    pub fst_data_byte: DataByte,
    pub snd_data_byte: DataByte,
}

#[derive(Debug, thiserror::Error)]
#[error("Sending MIDI Message failed. Reason: {human_friendly_description}.\n Details: {underlying_error:?}")]
pub struct SendFailed<'a> {
    pub human_friendly_description: &'a str,
    pub underlying_error: Option<Box<dyn error::Error>>,
}

pub trait MidiSender {
    fn send(&self, msg: MidiMessage) -> Result<(), SendFailed>;
}
