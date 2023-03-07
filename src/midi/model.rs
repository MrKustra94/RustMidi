use crate::extension::OptionExt;

use thiserror;

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(try_from = "u8")]
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

impl TryFrom<u8> for Status {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Status::from_u8(value).ok_or(format!(
            "Expected status to be between 128 and 255. Got {value}"
        ))
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct DataByte(u8);

impl DataByte {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn from_u8(db: u8) -> Option<DataByte> {
        Option::when(db & U8_MSB_EXTRACTOR == 0, || DataByte(db))
    }
}

impl TryFrom<u8> for DataByte {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        DataByte::from_u8(value).ok_or(format!(
            "Expected data byte to be between 0 and 127. Got {value}."
        ))
    }
}

#[derive(Debug)]
pub struct MidiMessage {
    pub status: Status,
    pub fst_data_byte: DataByte,
    pub snd_data_byte: DataByte,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct MidiSendFailed(#[from] pub anyhow::Error);

pub trait MidiSender {
    fn send(&self, msg: MidiMessage) -> Result<(), MidiSendFailed>;

    fn send_and_forget(&self, msg: MidiMessage) {
        let _ = self.send(msg);
    }
}
