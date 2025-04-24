use log::info;
use shared::broadcast_actions::{
    BroadcastActionsReply, BroadcastActionsRequest, broadcast_actions_server::BroadcastActions,
};
use shared::serialize::map_vec;
use state::{ReversibleAction, Store};
use std::marker::Send;
use std::sync::{Arc, Mutex, mpsc::channel};
use tonic::async_trait;

/// Context for collaborative editing.
/// So far, just contains a state store which is updated when the client broadcasts actions.
pub struct CollabContext {
    // Server's representation of the state store. Includes an undo stack which tracks which
    // actions have been processed so far.
    // TODO: this should be its own thing, not just a copy of the client side store.
    store: Arc<Mutex<Store>>,
    // TODO: reflect client undos/redos on the server. This can get messy when each client only
    // wants to undo its own changes! The naive solution will work most of the time but sometimes
    // we'll need smarter conflict resolution.
}

impl CollabContext {
    pub fn new(broadcast: impl Fn(Vec<ReversibleAction>) + Send + 'static) -> Self {
        // TODO: this is an example of why we don't need a whole Store here.
        // Maybe just a StoreData?
        let (tx, _rx) = channel();

        CollabContext {
            store: Arc::new(Mutex::new(Store::new(broadcast, tx))),
        }
    }
}

#[async_trait]
impl BroadcastActions for CollabContext {
    async fn broadcast_actions(
        &self,
        request: tonic::Request<BroadcastActionsRequest>,
    ) -> Result<tonic::Response<BroadcastActionsReply>, tonic::Status> {
        let actions: Vec<ReversibleAction> = map_vec(request.into_inner().actions);

        for action in actions {
            self.store
                .lock()
                .unwrap()
                .dispatch(&action.selector, action.forward);
            // TODO: validate that reverse actions match up - if not, then we need conflict resolution.
        }

        self.store.lock().unwrap().snapshot();

        info!("{:?}", self.store.lock().unwrap().get());

        Ok(tonic::Response::new(BroadcastActionsReply {
            success: true,
        }))
    }
}
