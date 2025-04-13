use shared::serialize::map_vec;
use state::broadcast_actions::{
    BroadcastActionsReply, BroadcastActionsRequest, broadcast_actions_server::BroadcastActions,
};
use state::{ReversibleAction, Store};
use std::sync::{Arc, Mutex};
use tonic::async_trait;

/// Context for collaborative editing.
/// For now, just
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
    pub fn new() -> Self {
        CollabContext {
            store: Arc::new(Mutex::new(Store::default())),
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

        Ok(tonic::Response::new(BroadcastActionsReply {
            success: true,
        }))
    }
}
