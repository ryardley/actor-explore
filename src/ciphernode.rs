use tokio::sync::mpsc;

use crate::{
    actor_traits::{Actor, ActorHandle},
    event::EnclaveEvent,
    event_dispatcher::EventDispatcher,
};
use async_trait::*;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Ciphernode {
    sender: mpsc::Sender<EnclaveEvent>,
}

impl Ciphernode {
    pub fn new(dispatcher: EventDispatcher) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(CiphernodeActor::new(receiver, dispatcher).run());
        Ciphernode { sender }
    }
}

#[async_trait]
impl ActorHandle<EnclaveEvent> for Ciphernode {
    async fn send(&self, event: EnclaveEvent) -> Result<()> {
        Ok(self.sender.send(event).await?)
    }
}

struct CiphernodeActor {
    receiver: mpsc::Receiver<EnclaveEvent>,
    dispatcher: EventDispatcher,
}

impl CiphernodeActor {
    pub fn new(receiver: mpsc::Receiver<EnclaveEvent>, dispatcher: EventDispatcher) -> Self {
        CiphernodeActor {
            receiver,
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
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await.unwrap()
        }
    }
}
