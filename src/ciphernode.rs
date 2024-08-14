use crate::{
    actor_traits::{Actor, ActorSender},
    encryptor::{Encryptor, Plaintext},
    event::EnclaveEvent,
    event_dispatcher::EventDispatcher,
    fhe::{Fhe, Rng},
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
    pub fn new<D, S, R, E>(dispatcher: D, store: S, fhe: Fhe<R>, encryptor: E) -> Self
    where
        S: Store,
        D: EventDispatcher<EnclaveEvent>,
        R: Rng,
        E: Encryptor,
    {
        let actor = CiphernodeActor::new(dispatcher, store, fhe, encryptor);
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

struct CiphernodeActor<S: Store, D: EventDispatcher<EnclaveEvent>, R: Rng, E: Encryptor> {
    dispatcher: D,
    store: S,
    fhe: Fhe<R>,
    encryptor: E,
}

impl<S, D, R, E> CiphernodeActor<S, D, R, E>
where
    S: Store,
    D: EventDispatcher<EnclaveEvent>,
    R: Rng,
    E: Encryptor,
{
    pub fn new(dispatcher: D, store: S, fhe: Fhe<R>, encryptor: E) -> Self {
        Self {
            dispatcher,
            store,
            fhe,
            encryptor,
        }
    }

    async fn on_computation_requested(&mut self, e3_id: &str) -> Result<()> {
        let (sk, pk) = self.fhe.generate_keyshare()?;
        let e_sk = self.encryptor.encrypt(Plaintext::new(sk.into())).await?;
        
        self.store.insert(format!("{}/sk",e3_id), e_sk);
        self.store.insert(format!("{}/pk",e3_id), pk.clone());
        
        let _ = self
            .dispatcher
            .send(EnclaveEvent::KeyshareCreated {
                e3_id: e3_id.to_string(),
                keyshare: pk,
            })
            .await;
        Ok(())
    }
}

#[async_trait]
impl<S, D, R, E> Actor<EnclaveEvent> for CiphernodeActor<S, D, R, E>
where
    S: Store,
    D: EventDispatcher<EnclaveEvent>,
    R: Rng,
    E: Encryptor,
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
