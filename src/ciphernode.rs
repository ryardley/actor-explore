use std::sync::Arc;

use crate::{
    actor_traits::{Actor, ActorHandle, ActorRunner, ActorSender},
    event::EnclaveEvent,
    event_dispatcher::EventDispatcher,
};
use async_trait::*;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug,Clone)]
pub struct Ciphernode(Arc<ActorHandle<EnclaveEvent>>);

impl Ciphernode {
    pub fn new(dispatcher: EventDispatcher) -> Self {
        let actor = CiphernodeActor::new(dispatcher);
        let runner = ActorRunner::new(actor, 8);
        Ciphernode(Arc::new(runner.handle()))
    }
}
#[async_trait]
impl ActorSender<EnclaveEvent> for Ciphernode {
    async fn send(&self, msg: EnclaveEvent) -> Result<()> {
        Ok(self.0.send(msg).await?)
    }
}
struct CiphernodeActor {
    dispatcher: EventDispatcher,
}

impl CiphernodeActor {
    pub fn new(dispatcher: EventDispatcher) -> Self {
        Self {
            dispatcher,
        }
    }

    async fn on_computation_requested(&mut self, e3_id: &str) -> Result<()> {
        let _ = self
            .dispatcher
            .send(EnclaveEvent::KeyshareCreated {
                e3_id: e3_id.to_string(),
                keyshare: "Hello World".to_string(),
            })
            .await;
        Ok(())
    }
}

#[async_trait]
impl Actor<EnclaveEvent> for CiphernodeActor {
    async fn handle_message(&mut self, msg: EnclaveEvent) -> Result<()> {
        match msg {
            EnclaveEvent::ComputationRequested { e3_id, .. } => {
                self.on_computation_requested(&e3_id).await?
            }
            _ => (),
        }
        Ok(())
    }
}
