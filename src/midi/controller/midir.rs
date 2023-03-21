use anyhow::anyhow;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel as cch;

use crate::midi::model::{MidiMessage, MidiSendFailed, MidiSender};
use crate::midi_model::{DataByte, MidiReceiver, Status};

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

pub struct MidirBasedSender {
    sender: cch::Sender<MidiMessage>,
    _sending_loop: JoinHandle<()>,
}

impl MidirBasedSender {
    pub fn new(controller: &str) -> anyhow::Result<MidirBasedSender> {
        let mut midi_out = Self::prepare_midi_out_connection(controller)?;
        let (sender, receiver) = cch::unbounded();
        let _sending_loop = thread::spawn(move || {
            while let Ok(midi_msg) = receiver.recv() {
                let bytes: [u8; 3] = (AsU8s(midi_msg)).into();
                // Fire and forget.
                let _ = midi_out.send(bytes.as_slice());
            }
        });

        Ok(MidirBasedSender {
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

impl MidiSender for MidirBasedSender {
    fn send(&self, msg: MidiMessage) -> Result<(), MidiSendFailed> {
        self.sender.send(msg).map_err(|e| MidiSendFailed(e.into()))
    }
}

pub struct MidirBasedReceiver {
    receiver: cch::Receiver<MidiMessage>,
    #[warn(unused_variables)]
    _connection: Mutex<midir::MidiInputConnection<()>>,
}

impl MidirBasedReceiver {
    pub fn new(controller: &str) -> anyhow::Result<MidirBasedReceiver> {
        let midi_input = midir::MidiInput::new(&format!("{controller}-client"))?;
        let input_port = Self::prepare_midi_in_port(controller, &midi_input)?;

        let (sender, receiver) = cch::unbounded();

        let _connection = Mutex::new(
            midi_input
                .connect(
                    &input_port,
                    controller,
                    move |_: u64, message: &[u8], _: &mut ()| {
                        let status = message[0];
                        let fst_db = message[1];
                        let snd_db = message[2];

                        let midi_msg = unsafe {
                            MidiMessage {
                                status: Status::from_u8_unsafe(status),
                                fst_data_byte: DataByte::from_u8_unsafe(fst_db),
                                snd_data_byte: DataByte::from_u8_unsafe(snd_db),
                            }
                        };

                        let _ = sender.send(midi_msg);
                    },
                    (),
                )
                .map_err(|e| anyhow!("Failed connecting to MIDI Input Device. Reason: {e}"))?,
        );

        Ok(MidirBasedReceiver {
            receiver,
            _connection,
        })
    }

    fn prepare_midi_in_port(
        controller: &str,
        midi_input: &midir::MidiInput,
    ) -> anyhow::Result<midir::MidiInputPort> {
        Self::set_up_port(midi_input, controller).ok_or_else(|| {
            anyhow::Error::msg(format!("Couldn't set up connection with {controller}."))
        })
    }

    fn set_up_port(mo: &midir::MidiInput, controller: &str) -> Option<midir::MidiInputPort> {
        mo.ports()
            .iter()
            .find_map(|p| {
                mo.port_name(p)
                    .ok()
                    .filter(|pn| pn.contains(controller))
                    .map(|_| p)
            })
            .cloned()
    }
}

impl MidiReceiver for MidirBasedReceiver {
    fn poll(&self) -> Option<MidiMessage> {
        self.receiver.recv().ok()
    }
}
