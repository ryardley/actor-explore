use crate::{
    actor_traits::{Actor, ActorHandle, ActorRunner, ActorSender},
    ciphernode::Ciphernode,
    event::EnclaveEvent,
    logger::Logger,
};
use async_trait::*;

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

#[derive(Debug, Clone)]
pub struct EventDispatcher(ActorHandle<EnclaveEvent>);

impl EventDispatcher {
    pub fn new() -> Self {
        let actor = EventDispatcherActor::new();
        let runner = ActorRunner::new(actor, 8);
        EventDispatcher(runner.handle())
    }

    pub async fn register(&self, listener: Listener) {
        let _ = self.0.send(EnclaveEvent::RegisterListener(listener)).await;
    }
}

#[async_trait]
impl ActorSender<EnclaveEvent> for EventDispatcher {
    async fn send(&self, msg: EnclaveEvent) -> Result<()> {
        Ok(self.0.send(msg).await?)
    }
}

struct EventDispatcherActor {
    listeners: Vec<Listener>,
}

impl EventDispatcherActor {
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
}
