use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::midi::model::{DataByte, MidiMessage, MidiSender, Status};

#[derive(Debug, serde::Deserialize)]
pub struct ColorMapping {
    pub ok: DataByte,
    pub action_triggerred: DataByte,
    pub transient_error: DataByte,
    pub not_ok: DataByte,
    pub initial: DataByte,
}

#[derive(Debug, serde::Deserialize)]
pub struct PadId {
    pub status: Status,
    pub fst_data_byte: DataByte,
}

pub struct PadMapping {
    pub pad_id: PadId,
    pub color_mapping: Arc<ColorMapping>,
}

impl PadMapping {
    pub fn custom_message(&self, snd_data_byte: DataByte) -> MidiMessage {
        self.prepare_message(snd_data_byte)
    }

    pub fn ok_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.ok)
    }

    pub fn transient_error_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.transient_error)
    }

    pub fn not_ok_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.not_ok)
    }

    pub fn action_triggerred_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.action_triggerred)
    }

    pub fn initial_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.initial)
    }

    fn prepare_message(&self, snd_data_byte: DataByte) -> MidiMessage {
        MidiMessage {
            status: self.pad_id.status,
            fst_data_byte: self.pad_id.fst_data_byte,
            snd_data_byte,
        }
    }
}

pub enum PadOutput {
    Ok,
    NotOk,
    TempError,
    Custom(DataByte),
}

#[async_trait::async_trait]
pub trait PadHandler {
    async fn handle(&self) -> PadOutput;
}

pub struct Config {
    pub pad_mapping: PadMapping,
    pub schedule_every: Duration,
}

pub struct PadActor {
    pub running_loop: tokio::task::JoinHandle<()>,
}

pub trait Runtime {
    fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn schedule_once<A, F>(&self, after: Duration, action: F)
    where
        A: Send + Sync + 'static,
        F: Future<Output = anyhow::Result<A>> + Send + Sync + 'static;
}

pub struct TokioRuntime {
    runtime: Arc<tokio::runtime::Runtime>,
}

impl TokioRuntime {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>) -> TokioRuntime {
        TokioRuntime { runtime }
    }
}

impl Runtime for TokioRuntime {
    fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(task)
    }

    fn schedule_once<A, F>(&self, after: Duration, action: F)
    where
        A: Send + Sync + 'static,
        F: Future<Output = anyhow::Result<A>> + Send + Sync + 'static,
    {
        tokio::spawn(async move {
            tokio::time::sleep(after).await;
            let _ = action.await;
        });
    }
}

pub trait ColorMappingProvider {
    fn provide_mapping(&self, status: &Status, fst_db: &DataByte) -> &ColorMapping;
}

impl PadActor {
    pub fn start<R: Runtime + Send + Sync + 'static>(
        handler: Arc<dyn PadHandler + Send + Sync>,
        midi_sender: Arc<dyn MidiSender + Send + Sync>,
        runtime: Arc<R>,
        config: Config,
    ) -> PadActor {
        let (msg_queue_sender, msg_queue_receiver) = async_channel::unbounded::<()>();
        let shared_queue_sender = Arc::new(msg_queue_sender);

        // Send WHITE color message. This should be treated as notification that MIDI pad is loaded correctly.
        let initial_msg = config.pad_mapping.initial_message();
        midi_sender.send_and_forget(initial_msg);

        let running_loop = runtime.clone().spawn(async move {
            // Send first message to initiate actor.
            let _ = shared_queue_sender.send(()).await;

            while msg_queue_receiver.recv().await.is_ok() {
                // Send action triggered message. That will signal that action has been initiated.
                let pending_msg = config.pad_mapping.action_triggerred_message();
                midi_sender.send_and_forget(pending_msg);

                // Send message based on handler output.
                let result_msg = match handler.handle().await {
                    PadOutput::Ok => config.pad_mapping.ok_message(),
                    PadOutput::NotOk => config.pad_mapping.not_ok_message(),
                    PadOutput::TempError => config.pad_mapping.transient_error_message(),
                    PadOutput::Custom(message) => config.pad_mapping.custom_message(message),
                };
                midi_sender.send_and_forget(result_msg);

                let loop_queue_sender = shared_queue_sender.clone();
                runtime.schedule_once(config.schedule_every, async move {
                    loop_queue_sender.send(()).await?;
                    Ok(())
                });
            }
        });

        PadActor { running_loop }
    }
}
