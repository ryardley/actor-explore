use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::{actor_traits::*, event::EnclaveEvent};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub enum LogEvent {
    Log(EnclaveEvent),
    GetLog(oneshot::Sender<Vec<EnclaveEvent>>),
}

#[derive(Clone, Debug)]
pub struct Logger {
    sender: mpsc::Sender<LogEvent>,
}

impl Logger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(LoggerActor::new(receiver).run());
        Self { sender }
    }
    pub async fn get_log(&self) -> Result<Vec<EnclaveEvent>> {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(LogEvent::GetLog(send)).await;
        Ok(recv.await?)
    }
}

#[async_trait]
impl ActorHandle<EnclaveEvent> for Logger {
    async fn send(&self, event: EnclaveEvent) -> Result<()> {
        let _ = self.sender.send(LogEvent::Log(event)).await;
        Ok(())
    }
}

#[derive(Debug)]
struct LoggerActor {
    receiver: mpsc::Receiver<LogEvent>,
    log: Vec<EnclaveEvent>,
}
impl LoggerActor {
    pub fn new(receiver: mpsc::Receiver<LogEvent>) -> Self {
        Self {
            log: vec![],
            receiver,
        }
    }
}

#[async_trait]
impl Actor<LogEvent> for LoggerActor {
    async fn handle_message(&mut self, msg: LogEvent) -> Result<()> {
        match msg {
            LogEvent::Log(log_msg) => self.log.push(log_msg),
            LogEvent::GetLog(sender) => {
                let _ = sender.send(self.log.clone());
            }
        };
        Ok(())
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap()
        }
    }
}
