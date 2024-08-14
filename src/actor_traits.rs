use std::marker::PhantomData;

use async_trait::async_trait;
use tokio::sync::mpsc;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// Actor trait
/// This defines the Actor that handles messages.
#[async_trait]
pub trait Actor<M:Send + 'static>:Send + 'static {
    async fn handle_message(&mut self, msg: M) -> Result<()>;
}

/// ActorHandle trait
/// The object that can send messages to the actor. Think of this as the external API of the
/// Actor
#[async_trait]
pub trait ActorSender<M> {
    async fn send(&self, msg: M) -> Result<()>;
}

//
pub struct ActorRunner<A, M>
where
    A: Actor<M>,
    M: Send + 'static,
{
    sender: mpsc::Sender<M>,
    _phantom: PhantomData<A>,
}

impl<A, M> ActorRunner<A, M>
where
    A: Actor<M>,
    M: Send + 'static,
{
    pub fn new(actor: A, buffer: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel(buffer);

        tokio::spawn(async move {
            let mut actor = actor;
            while let Some(msg) = receiver.recv().await {
                if let Err(e) = actor.handle_message(msg).await {
                    eprintln!("Error handling message: {:?}", e);
                }
            }
        });

        Self {
            sender,
            _phantom: PhantomData,
        }
    }

    pub fn handle(&self) -> ActorHandle<M> {
        ActorHandle {
            sender: self.sender.clone(),
        }
    }
}

#[async_trait]
impl<M: Send> ActorSender<M> for ActorHandle<M> {
    async fn send(&self, msg: M) -> Result<()> {
        let _ = self.sender.send(msg).await;
        Ok(())
    }
}

// ActorHandle struct
#[derive(Clone,Debug)]
pub struct ActorHandle<M> {
    sender: mpsc::Sender<M>,
}
