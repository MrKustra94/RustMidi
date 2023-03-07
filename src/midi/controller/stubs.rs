use crate::midi::model::{MidiMessage, MidiSendFailed, MidiSender};

pub struct JustPrint;

impl MidiSender for JustPrint {
    fn send(&self, msg: MidiMessage) -> Result<(), MidiSendFailed> {
        println!("Sent: {msg:?}");
        Ok(())
    }
}
