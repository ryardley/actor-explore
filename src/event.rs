use crate::event_dispatcher::Listener;

// type Error = Box<dyn std::error::Error>;
// type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum EnclaveEvent {
    RegisterListener(Listener),
    ComputationRequested {
        e3_id: String,
        // computation_type: ??, // TODO:
        // execution_model_type: ??, // TODO:
        ciphernode_group_length: u32,
        ciphernode_threshold: u32,
        // input_deadline: ??, // TODO:
        // availability_duration: ??, // TODO:
        sortition_seed: u32,
    },
    KeyshareCreated {
        e3_id: String,
        keyshare: String,
    },
}
