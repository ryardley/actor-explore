use async_trait::async_trait;
use tokio::sync::mpsc::{self, Receiver};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// Actor trait
/// This defines the Actor that handles messages.
#[async_trait]
pub trait Actor<M: Send + 'static>: Send + 'static {
    async fn handle_message(&mut self, msg: M) -> Result<()>;
}

/// ActorHandle trait
/// The object that can send messages to the actor. Think of this as the external API of the
/// Actor
#[async_trait]
pub trait ActorSender<M> {
    async fn send(&self, msg: M) -> Result<()>;
}

pub fn run_actor<A, M>(actor: A, buffer: usize) -> mpsc::Sender<M>
where
    A: Actor<M>,
    M: Send + 'static
{
    let (sender, receiver) = mpsc::channel(buffer);

    tokio::spawn(consume_actor(actor, receiver));

    sender
}

async fn consume_actor<A, M>(mut actor: A, mut receiver: Receiver<M>)
where
    A: Actor<M>,
    M: Send + 'static,
{
    while let Some(msg) = receiver.recv().await {
        if let Err(e) = actor.handle_message(msg).await {
            eprintln!("Error handling message: {:?}", e);
        }
    }
}
