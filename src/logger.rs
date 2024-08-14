use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::{actor_traits::*, event::EnclaveEvent};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum LogEvent {
    Log(EnclaveEvent),
    GetLog(oneshot::Sender<Vec<EnclaveEvent>>),
}

#[derive(Debug, Clone)]
pub struct Logger {
    sender: mpsc::Sender<LogEvent>,
}

impl Logger {
    pub fn new() -> Self {
        let actor = LoggerActor::new();
        let sender = run_actor(actor, 8);
        Logger { sender }
    }
    pub async fn get_log(&self) -> Result<Vec<EnclaveEvent>> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(LogEvent::GetLog(send)).await;
        Ok(recv.await?)
    }
}

#[async_trait]
impl ActorSender<EnclaveEvent> for Logger {
    async fn send(&self, msg: EnclaveEvent) -> Result<()> {
        Ok(self.sender.send(LogEvent::Log(msg)).await?)
    }
}

struct LoggerActor {
    log: Vec<EnclaveEvent>,
}

impl LoggerActor {
    pub fn new() -> Self {
        Self { log: vec![] }
    }
}

#[async_trait]
impl Actor<LogEvent> for LoggerActor {
    async fn handle_message(&mut self, msg: LogEvent) -> Result<()> {
        match msg {
            LogEvent::Log(log_msg) => self.log.push(log_msg),
            LogEvent::GetLog(reply) => {
                let _ = reply.send(self.log.clone());
            }
        }
        Ok(())
    }
}
