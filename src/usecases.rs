use std::sync::{Arc, Mutex};

use crate::{
    data::Store,
    encryptor::Encryptor,
    events::{EnclaveEvent, EventProducer, KeyshareCreated},
    fhe::{Fhe, Rng, SecretKey},
};

// Some loose error/result stuff we can use
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub async fn create_and_store_keyshare<R: Rng, P: EventProducer>(
    fhe: &Fhe,
    rng: Arc<Mutex<R>>,
    store: &mut impl Store,
    encryptor: &impl Encryptor<SecretKey>,
    producer: &P,
    e3_id: &str,
) -> Result<()> {
    // create FHE secret keypair
    let (sk, pk) = fhe.generate_keyshare(rng)?;
    
    // encrypt secret at rest
    let e_sk = encryptor.encrypt(sk)?;
    
    // save encrypted secret key
    store.insert(&format!("{}/sk", e3_id).into_bytes(), &e_sk.as_bytes())?;

    // save public key
    store.insert(&format!("{}/pk", e3_id).into_bytes(), &pk.as_bytes())?;

    // dispatch KeyshareCreated
    producer
        .emit(EnclaveEvent::KeyshareCreated(KeyshareCreated {
            pubkey: pk,
        }))
        .await?;

    Ok(())
}

async fn decrypt_output_share<Ct>(_store: impl Store, _e3_id: &str, _ciphertext: Ct) -> Result<()> {
    todo!();
}

async fn destroy_keyshare(_e3_id: impl Into<String>) -> Result<()> {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::{
        data::MockStore,
        encryptor::{Encrypted, MockEncryptor},
        events::MockEventProducer,
        fhe,
    };
    use mockall::predicate::{eq, function};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;

    // Some loose error/result stuff we can use
    pub type Error = Box<dyn std::error::Error>;
    pub type Result<T> = std::result::Result<T, Error>;

    fn test_rng() -> Arc<Mutex<ChaCha8Rng>> {
        Arc::new(Mutex::new(ChaCha8Rng::seed_from_u64(42)))
    }

    #[tokio::test]
    // #[ignore]
    async fn test_create_and_store_keyshare() -> Result<()> {
        // using concrete Fhe but creating inflection point
        let fhe = Fhe::new(test_rng(), vec![0x3FFFFFFF000001], 2048, 1032193)?;

        let mut encryptor = MockEncryptor::<fhe::SecretKey>::new();
        let mut store = MockStore::new();
        let mut producer = MockEventProducer::new();

        let shared_rng = test_rng();
        let (exp_sk, exp_pk) = fhe.generate_keyshare(shared_rng)?;

        let encrypted_sk = Encrypted::<SecretKey>::new(vec![1, 2, 3, 4]);

        encryptor
            .expect_encrypt()
            .with(function(move |sk: &SecretKey| sk.eq(&exp_sk)))
            .return_once(move |_| Ok(encrypted_sk.clone()));

        store
            .expect_insert()
            .with(eq(b"myid/sk".to_vec()), eq(vec![1, 2, 3, 4]))
            .times(1)
            .return_once(|_, __| Ok(Some(vec![])));

        store
            .expect_insert()
            .with(eq(b"myid/pk".to_vec()), eq(exp_pk.as_bytes().to_vec()))
            .times(1)
            .return_once(|_, __| Ok(Some(vec![])));

        producer.expect_emit().times(1).return_once(|_| Ok(()));

        create_and_store_keyshare(&fhe, test_rng(), &mut store, &encryptor, &producer, "myid")
            .await?;

        Ok(())
    }
}
