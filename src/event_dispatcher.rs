use crate::{
    actor_traits::{run_actor, Actor, ActorSender},
    ciphernode::Ciphernode,
    event::EnclaveEvent,
    logger::Logger,
};
use async_trait::*;
use tokio::sync::mpsc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Listener {
    Ciphernode(Ciphernode),
    Reporter(Logger),
}

#[async_trait]
impl ActorSender<EnclaveEvent> for Listener {
    async fn send(&self, event: EnclaveEvent) -> Result<()> {
        Ok(match self {
            Listener::Ciphernode(c) => c.send(event).await?,
            Listener::Reporter(c) => c.send(event).await?,
        })
    }
}

#[async_trait]
pub trait EventDispatcher<E>: ActorSender<E> + Send + 'static {
    async fn register(&self, listener: Listener);
}

#[derive(Debug, Clone)]
pub struct EventBus {
    sender: mpsc::Sender<EnclaveEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let actor = EventBusActor::new();
        let sender = run_actor(actor, 8);
        EventBus { sender }
    }
}

#[async_trait]
impl EventDispatcher<EnclaveEvent> for EventBus {
    async fn register(&self, listener: Listener) {
        let _ = self.send(EnclaveEvent::RegisterListener(listener)).await;
    }
}

#[async_trait]
impl ActorSender<EnclaveEvent> for EventBus {
    async fn send(&self, msg: EnclaveEvent) -> Result<()> {
        Ok(self.sender.send(msg).await?)
    }
}

struct EventBusActor {
    listeners: Vec<Listener>,
}

impl EventBusActor {
    pub fn new() -> Self {
        Self { listeners: vec![] }
    }

    async fn dispatch(&self, event: EnclaveEvent) -> Result<()> {
        for listener in self.listeners.iter() {
            listener.send(event.clone()).await?
        }
        Ok(())
    }
}

#[async_trait]
impl Actor<EnclaveEvent> for EventBusActor {
    async fn handle_message(&mut self, msg: EnclaveEvent) -> Result<()> {
        match msg {
            EnclaveEvent::RegisterListener(listener) => self.listeners.push(listener),
            other => {
                let _ = self.dispatch(other).await;
            }
        }
        Ok(())
    }
}
