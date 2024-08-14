use std::fmt::Debug;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Key,
};
use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::actor_traits::*;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

// TODO: Zeroize etc
pub struct Plaintext(Vec<u8>);

impl Plaintext {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
}

impl From<Vec<u8>> for Plaintext {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<Plaintext> for Vec<u8> {
    fn from(plaintext: Plaintext) -> Self {
        plaintext.0
    }
}

pub enum EncryptorMessage {
    Encrypt {
        plaintext: Plaintext,
        reply: oneshot::Sender<Vec<u8>>,
    },
}

#[async_trait]
pub trait Encryptor: Send + 'static {
    async fn encrypt(&self, plaintext: Plaintext) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub struct AesEncryptor {
    sender: mpsc::Sender<EncryptorMessage>,
}

impl AesEncryptor {
    pub fn new(key: Vec<u8>) -> Self {
        let actor = EncryptorActor::new(key);
        let sender = run_actor(actor, 8);
        AesEncryptor { sender }
    }
}

#[async_trait]
impl Encryptor for AesEncryptor {
    async fn encrypt(&self, plaintext: Plaintext) -> Result<Vec<u8>> {
        let (send, recv) = oneshot::channel();
        let _ = self
            .sender
            .send(EncryptorMessage::Encrypt {
                plaintext,
                reply: send,
            })
            .await;
        Ok(recv.await?)
    }
}

struct EncryptorActor {
    key: Vec<u8>,
}

impl EncryptorActor {
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    fn encrypt(&self, data: Plaintext) -> Result<Vec<u8>> {
        let serialized: Vec<u8> = data.into();
        let k = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(k);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, serialized.as_ref())
            .expect("Encryption failed"); // TODO: fix this when tidying up errors
        Ok(ciphertext)
    }
}

#[async_trait]
impl Actor<EncryptorMessage> for EncryptorActor {
    async fn handle_message(&mut self, msg: EncryptorMessage) -> Result<()> {
        match msg {
            EncryptorMessage::Encrypt { reply, plaintext } => {
                let encrypted = self.encrypt(plaintext)?;
                let _ = reply.send(encrypted);
            }
        }
        Ok(())
    }
}
