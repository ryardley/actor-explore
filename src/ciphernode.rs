use crate::{
    actor_traits::{Actor, ActorSender},
    event::EnclaveEvent,
    event_dispatcher::EventDispatcher,
    run_actor,
    store::Store,
};
use async_trait::*;
use tokio::sync::mpsc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Ciphernode {
    sender: mpsc::Sender<EnclaveEvent>,
}

impl Ciphernode {
    pub fn new<E, S>(dispatcher: E, store: S) -> Self
    where
        S: Store,
        E: EventDispatcher<EnclaveEvent>,
    {
        let actor = CiphernodeActor::new(dispatcher, store);
        let sender = run_actor(actor, 8);
        Ciphernode { sender }
    }
}

#[async_trait]
impl ActorSender<EnclaveEvent> for Ciphernode {
    async fn send(&self, msg: EnclaveEvent) -> Result<()> {
        Ok(self.sender.send(msg).await?)
    }
}

struct CiphernodeActor<S: Store, E: EventDispatcher<EnclaveEvent>> {
    dispatcher: E,
    store: S,
}

impl<S, E> CiphernodeActor<S, E>
where
    S: Store,
    E: EventDispatcher<EnclaveEvent>,
{
    pub fn new(dispatcher: E, store: S) -> Self {
        Self { dispatcher, store }
    }

    async fn on_computation_requested(&mut self, e3_id: &str) -> Result<()> {
        self.store.insert(vec![123, 12]);
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
impl<S, E> Actor<EnclaveEvent> for CiphernodeActor<S, E>
where
    S: Store,
    E: EventDispatcher<EnclaveEvent>,
{
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
