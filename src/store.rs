use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::actor_traits::*;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum StoreEvent {
    Insert(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct DataStore {
    sender: mpsc::Sender<StoreEvent>,
}

pub trait Store: Send + 'static {
    fn insert(&self, data: impl Into<Vec<u8>>);
}

impl DataStore {
    pub fn new() -> Self {
        let actor = StoreActor::new();
        let sender = run_actor(actor, 8);
        DataStore { sender }
    }
}
impl Store for DataStore {
    fn insert(&self, data: impl Into<Vec<u8>>) {
        let _ = self.sender.send(StoreEvent::Insert(data.into()));
    }
}

struct StoreActor;

impl StoreActor {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Actor<StoreEvent> for StoreActor {
    async fn handle_message(&mut self, msg: StoreEvent) -> Result<()> {
        match msg {
            StoreEvent::Insert(_) => (),
        }
        Ok(())
    }
}
