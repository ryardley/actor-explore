use crate::{
    actor_traits::{Actor, ActorHandle},
    ciphernode::Ciphernode,
    event::EnclaveEvent,
    logger::Logger,
};
use async_trait::*;
use tokio::sync::mpsc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Listener {
    Ciphernode(Ciphernode),
    Reporter(Logger),
}

#[async_trait]
impl ActorHandle<EnclaveEvent> for Listener {
    async fn send(&self, event: EnclaveEvent) -> Result<()> {
        Ok(match self {
            Listener::Ciphernode(c) => c.send(event).await?,
            Listener::Reporter(c) => c.send(event).await?,
        })
    }
}

struct EventDispatcherActor {
    receiver: mpsc::Receiver<EnclaveEvent>,
    listeners: Vec<Listener>,
}

impl EventDispatcherActor {
    pub fn new(receiver: mpsc::Receiver<EnclaveEvent>) -> Self {
        Self {
            receiver,
            listeners: vec![],
        }
    }

    async fn dispatch(&self, event: EnclaveEvent) -> Result<()> {
        for listener in self.listeners.iter() {
            listener.send(event.clone()).await?
        }
        Ok(())
    }
}

#[async_trait]
impl Actor<EnclaveEvent> for EventDispatcherActor {
    async fn handle_message(&mut self, msg: EnclaveEvent) -> Result<()> {
        match msg {
            EnclaveEvent::RegisterListener(listener) => self.listeners.push(listener),
            other => {
                let _ = self.dispatch(other).await;
            }
        }
        Ok(())
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap()
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventDispatcher {
    sender: mpsc::Sender<EnclaveEvent>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(EventDispatcherActor::new(receiver).run());
        EventDispatcher { sender }
    }

    pub async fn register(&self, listener: Listener) {
        let _ = self.send(EnclaveEvent::RegisterListener(listener)).await;
    }
}

#[async_trait]
impl ActorHandle<EnclaveEvent> for EventDispatcher {
    async fn send(&self, event: EnclaveEvent) -> Result<()> {
        Ok(self.sender.send(event).await?)
    }
}
