mod actor_traits;
mod ciphernode;
mod encryptor;
mod event;
mod event_dispatcher;
mod fhe;
mod logger;
mod store;
// mod usecases;

use actor_traits::*;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use rand::{rngs::OsRng, SeedableRng};
    use rand_chacha::ChaCha20Rng;
    use tokio::time::sleep;

    use crate::{
        actor_traits::*,
        ciphernode::Ciphernode,
        encryptor::AesEncryptor,
        event::EnclaveEvent,
        event_dispatcher::{EventBus, EventDispatcher, Listener},
        fhe::Fhe,
        logger::Logger,
        store::DataStore,
    };

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[tokio::test]
    async fn test_main() -> Result<()> {
        let dispatcher = EventBus::new();
        let store = DataStore::new();
        let key = b"a 32-byte secret key here!!!!!!!".to_vec();
        let encryptor = AesEncryptor::new(key);
        let fhe = Fhe::new(
            Arc::new(Mutex::new(ChaCha20Rng::seed_from_u64(42))),
            vec![0x3FFFFFFF000001],
            2048,
            1032193,
        )?;

        let ciphernode1 = Ciphernode::new(
            dispatcher.clone(),
            store.clone(),
            fhe.clone(),
            encryptor.clone(),
        );
        let ciphernode2 = Ciphernode::new(
            dispatcher.clone(),
            store.clone(),
            fhe.clone(),
            encryptor.clone(),
        );
        let ciphernode3 = Ciphernode::new(
            dispatcher.clone(),
            store.clone(),
            fhe.clone(),
            encryptor.clone(),
        );
        let reporter = Logger::new();

        dispatcher
            .register(Listener::Reporter(reporter.clone()))
            .await;
        dispatcher.register(Listener::Ciphernode(ciphernode1)).await;
        dispatcher.register(Listener::Ciphernode(ciphernode2)).await;
        dispatcher.register(Listener::Ciphernode(ciphernode3)).await;
        dispatcher
            .send(EnclaveEvent::ComputationRequested {
                e3_id: "1234".to_string(),
                ciphernode_group_length: 3,
                ciphernode_threshold: 3,
                sortition_seed: 1234,
            })
            .await?;
        sleep(Duration::from_millis(0)).await;

        let log = reporter.get_log().await?;
        let expected = vec![
            EnclaveEvent::ComputationRequested {
                e3_id: "1234".to_owned(),
                ciphernode_group_length: 3,
                ciphernode_threshold: 3,
                sortition_seed: 1234,
            },
            // TODO: get correct test data....
            // EnclaveEvent::KeyshareCreated {
            //     e3_id: "1234".to_owned(),
            //     keyshare: "Hello World".to_owned(),
            // },
            // EnclaveEvent::KeyshareCreated {
            //     e3_id: "1234".to_owned(),
            //     keyshare: "Hello World".to_owned(),
            // },
            // EnclaveEvent::KeyshareCreated {
            //     e3_id: "1234".to_owned(),
            //     keyshare: "Hello World".to_owned(),
            // },
        ];

        assert_eq!(format!("{:?}", log), format!("{:?}", expected));

        Ok(())
    }
}
