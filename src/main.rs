mod actor_traits;
mod ciphernode;
mod event;
mod event_dispatcher;
mod logger;

use std::time::Duration;

use actor_traits::*;
use ciphernode::Ciphernode;
use event::EnclaveEvent;
use event_dispatcher::{EventDispatcher, Listener};
use logger::Logger;
use tokio::time::sleep;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let dispatcher = EventDispatcher::new();
    let ciphernode1 = Ciphernode::new(dispatcher.clone());
    let ciphernode2 = Ciphernode::new(dispatcher.clone());
    let reporter = Logger::new();

    dispatcher
        .register(Listener::Reporter(reporter.clone()))
        .await;
    dispatcher.register(Listener::Ciphernode(ciphernode1)).await;
    dispatcher.register(Listener::Ciphernode(ciphernode2)).await;
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
    for line in log {
        println!("{:?}", line);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::{
        actor_traits::*,
        ciphernode::Ciphernode,
        event::EnclaveEvent,
        event_dispatcher::{EventDispatcher, Listener},
        logger::Logger,
    };

    type Error = Box<dyn std::error::Error>;
    type Result<T> = std::result::Result<T, Error>;

    #[tokio::test]
    async fn test_main() -> Result<()> {
        let dispatcher = EventDispatcher::new();
        let ciphernode1 = Ciphernode::new(dispatcher.clone());
        let ciphernode2 = Ciphernode::new(dispatcher.clone());
        let reporter = Logger::new();

        dispatcher
            .register(Listener::Reporter(reporter.clone()))
            .await;
        dispatcher.register(Listener::Ciphernode(ciphernode1)).await;
        dispatcher.register(Listener::Ciphernode(ciphernode2)).await;
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
            EnclaveEvent::KeyshareCreated {
                e3_id: "1234".to_owned(),
                keyshare: "Hello World".to_owned(),
            },
            EnclaveEvent::KeyshareCreated {
                e3_id: "1234".to_owned(),
                keyshare: "Hello World".to_owned(),
            },
        ];
        assert_eq!(format!("{:?}",log), format!("{:?}", expected));

    
        Ok(())
    }
}
