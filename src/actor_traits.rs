use async_trait::*;
type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Actor<E> {
    async fn handle_message(&mut self, msg: E) -> Result<()>;
    async fn run(mut self);
}

#[async_trait]
pub trait ActorHandle<E>: Clone {
    async fn send(&self, event: E) -> Result<()>;
}

