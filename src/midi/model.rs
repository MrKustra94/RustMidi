use crate::extensions::option::OptionExt;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::error;
use std::fmt::Formatter;
use thiserror;

#[derive(Clone, Copy, Debug)]
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

struct StatusVisitor;

impl<'de> Visitor<'de> for StatusVisitor {
    type Value = Status;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Expecting status to be u8 between 0x80 and 0xFF.")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let parse_res = u8::try_from(v).ok().and_then(Status::from_u8);

        match parse_res {
            None => Err(E::custom(format!(
                "Expecting status to be u8 between 0x80 and 0xFF. Got: {}.",
                v
            ))),
            Some(status) => Ok(status),
        }
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u8(StatusVisitor)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
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

struct DataByteVisitor;

impl<'de> Visitor<'de> for DataByteVisitor {
    type Value = DataByte;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Expecting data byte to be u8 between 0x00 and 0x7F.")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let parse_res = u8::try_from(v).ok().and_then(DataByte::from_u8);

        match parse_res {
            None => Err(E::custom(format!(
                "Expecting data byte to be u8 between 0x00 and 0x7F. Got: {}.",
                v
            ))),
            Some(db) => Ok(db),
        }
    }
}

#[derive(Debug)]
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
