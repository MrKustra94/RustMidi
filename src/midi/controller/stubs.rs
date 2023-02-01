use crate::midi::model::SendFailed;
use crate::{MidiMessage, MidiSender};

pub struct JustPrint;

impl MidiSender for JustPrint {
    fn send(&self, msg: MidiMessage) -> Result<(), SendFailed> {
        println!("Sent: {:?}", msg);
        Ok(())
    }
}
