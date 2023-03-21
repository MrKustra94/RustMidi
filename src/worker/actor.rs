use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::midi::model::{DataByte, MidiMessage, MidiSender, Status};
use crate::midi_model::MidiReceiver;

#[derive(Debug, serde::Deserialize)]
pub struct ColorMapping {
    pub ok: DataByte,
    pub action_triggerred: DataByte,
    pub transient_error: DataByte,
    pub not_ok: DataByte,
    pub initial: DataByte,
    pub paused: DataByte,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, serde::Deserialize)]
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

    pub fn paused_message(&self) -> MidiMessage {
        self.prepare_message(self.color_mapping.paused)
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
pub trait PadHandler: Send + Sync {
    async fn handle(&mut self) -> PadOutput;
}

pub trait Runtime: Send + Sync + 'static {
    fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn spawn_blocking<F, R>(&self, task: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static;

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

    fn spawn_blocking<F, R>(&self, task: F) -> tokio::task::JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        self.runtime.spawn_blocking(task)
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

enum Command {
    TriggerHandler,
    PadPressed,
}

pub struct Config {
    pub pad_mapping: PadMapping,
    pub schedule_every: Duration,
}

pub struct ActorHandle(pub tokio::task::JoinHandle<()>);

#[derive(PartialEq, Eq)]
enum ActorStatus {
    Running,
    Stopped,
}

struct ActorCtx {
    handler: Arc<tokio::sync::Mutex<dyn PadHandler>>,
    midi_sender: Arc<dyn MidiSender + Send + Sync>,
    pad_mapping: PadMapping,
    status: ActorStatus,
}

impl ActorCtx {
    async fn handle(&mut self, command: Command) {
        match (&self.status, command) {
            (ActorStatus::Running, Command::TriggerHandler) => {
                // Send action triggered message. That will signal that action has been initiated.
                let pending_msg = self.pad_mapping.action_triggerred_message();
                self.midi_sender.send_and_forget(pending_msg);

                // Send message based on handler output.
                let mut lock = self.handler.lock().await;
                let result_msg = match lock.handle().await {
                    PadOutput::Ok => self.pad_mapping.ok_message(),
                    PadOutput::NotOk => self.pad_mapping.not_ok_message(),
                    PadOutput::TempError => self.pad_mapping.transient_error_message(),
                    PadOutput::Custom(message) => self.pad_mapping.custom_message(message),
                };
                self.midi_sender.send_and_forget(result_msg);
            }
            (ActorStatus::Running, Command::PadPressed) => {
                self.midi_sender
                    .send_and_forget(self.pad_mapping.paused_message());
                self.status = ActorStatus::Stopped
            }
            (ActorStatus::Stopped, Command::PadPressed) => {
                self.midi_sender
                    .send_and_forget(self.pad_mapping.initial_message());
                self.status = ActorStatus::Running
            }
            _ => {}
        }
    }
}

pub struct PadActor {
    sender: async_channel::Sender<Command>,
    pub pad_id: PadId,
}

impl PadActor {
    async fn send_pad_pressed(&self) {
        let _ = self.sender.send(Command::PadPressed).await;
    }

    pub fn start<R: Runtime>(
        handler: Arc<tokio::sync::Mutex<dyn PadHandler>>,
        midi_sender: Arc<dyn MidiSender + Send + Sync>,
        runtime: Arc<R>,
        config: Config,
    ) -> (ActorHandle, PadActor) {
        let (sender, msg_queue_receiver) = async_channel::unbounded::<Command>();

        // Send WHITE color message. This should be treated as notification that MIDI pad is loaded correctly.
        let initial_msg = config.pad_mapping.initial_message();
        let pad_id = config.pad_mapping.pad_id.clone();
        midi_sender.send_and_forget(initial_msg);

        let shared_queue_sender = Arc::new(sender.clone());
        let running_loop = runtime.clone().spawn(async move {
            // Send first message to initiate actor.
            let _ = shared_queue_sender.send(Command::TriggerHandler).await;

            let mut actor_ctx = ActorCtx {
                handler,
                midi_sender: midi_sender.clone(),
                pad_mapping: config.pad_mapping,
                status: ActorStatus::Running,
            };

            while let Ok(cmd) = msg_queue_receiver.recv().await {
                actor_ctx.handle(cmd).await;

                if actor_ctx.status == ActorStatus::Running {
                    let loop_queue_sender = shared_queue_sender.clone();
                    runtime.schedule_once(config.schedule_every, async move {
                        loop_queue_sender.send(Command::TriggerHandler).await?;
                        Ok(())
                    });
                }
            }
        });

        let actor = PadActor { sender, pad_id };
        let actor_handle = ActorHandle(running_loop);
        (actor_handle, actor)
    }
}

pub struct PadChangesListener {
    registered: Arc<dashmap::DashMap<PadId, Arc<PadActor>>>,
}

impl PadChangesListener {
    pub fn register(&self, pad_id: PadId, actor: Arc<PadActor>) {
        let _ = self.registered.insert(pad_id, actor);
    }

    pub fn start<MR, RT>(midi_receiver: MR, runtime: Arc<RT>) -> (ActorHandle, PadChangesListener)
    where
        MR: MidiReceiver + Send + Sync + 'static,
        RT: Runtime,
    {
        let registered: Arc<dashmap::DashMap<PadId, Arc<PadActor>>> =
            Arc::new(dashmap::DashMap::new());

        let loop_registered = registered.clone();
        let running_loop = runtime.clone().spawn(async move {
            let loop_mr = Arc::new(midi_receiver);

            while let Ok(Some(msg)) = {
                let iteration_mr = loop_mr.clone();
                runtime.spawn_blocking(move || iteration_mr.poll()).await
            } {
                let pad_id = PadId {
                    status: msg.status,
                    fst_data_byte: msg.fst_data_byte,
                };

                if let Some(actor) = loop_registered.get(&pad_id) {
                    if msg.snd_data_byte == unsafe { DataByte::from_u8_unsafe(127) } {
                        let pad_actor: &Arc<PadActor> = actor.value();
                        pad_actor.send_pad_pressed().await;
                    };
                }
            }
        });

        let actor = PadChangesListener { registered };
        let handle = ActorHandle(running_loop);
        (handle, actor)
    }
}
