use crossbeam_channel as cch;
use std::thread;
use std::thread::JoinHandle;

use crate::midi::model::{MidiMessage, MidiSendFailed, MidiSender};

struct AsU8s(MidiMessage);

impl From<AsU8s> for [u8; 3] {
    fn from(mm: AsU8s) -> Self {
        [
            mm.0.status.as_u8(),
            mm.0.fst_data_byte.as_u8(),
            mm.0.snd_data_byte.as_u8(),
        ]
    }
}

pub struct MidirBased {
    sender: cch::Sender<MidiMessage>,
    _sending_loop: JoinHandle<()>,
}

impl MidirBased {
    pub fn new(controller: &str) -> anyhow::Result<MidirBased> {
        let mut midi_out = Self::prepare_midi_out_connection(controller)?;
        let (sender, receiver) = cch::unbounded();
        let _sending_loop = thread::spawn(move || {
            while let Ok(midi_msg) = receiver.recv() {
                let bytes: [u8; 3] = (AsU8s(midi_msg)).into();
                // Fire and forget.
                let _ = midi_out.send(bytes.as_slice());
            }
        });

        Ok(MidirBased {
            sender,
            _sending_loop,
        })
    }

    fn prepare_midi_out_connection(
        controller: &str,
    ) -> anyhow::Result<midir::MidiOutputConnection> {
        let midi_port = midir::MidiOutput::new(&format!("{controller}-client"))?;
        Self::set_up_connection(midi_port, controller).ok_or_else(|| {
            anyhow::Error::msg(format!("Couldn't set up connection with {controller}."))
        })
    }

    fn set_up_connection(
        mo: midir::MidiOutput,
        controller: &str,
    ) -> Option<midir::MidiOutputConnection> {
        mo.ports()
            .iter()
            .find_map(|p| {
                mo.port_name(p)
                    .ok()
                    .filter(|pn| pn.contains(controller))
                    .map(|_| p)
            })
            .map(|p| mo.connect(p, controller).unwrap())
    }
}

impl MidiSender for MidirBased {
    fn send(&self, msg: MidiMessage) -> Result<(), MidiSendFailed> {
        self.sender.send(msg).map_err(|e| MidiSendFailed(e.into()))
    }
}
