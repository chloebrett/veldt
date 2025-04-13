use shared::serialize::map_vec;
use state::ReversibleAction;
use shared::consts::XERIC_URL;
use shared::model::Project;
use shared::broadcast_actions::{BroadcastActionsRequest, broadcast_actions_client::BroadcastActionsClient};
use tonic_web_wasm_client::Client;

pub async fn broadcast_actions(actions: Vec<ReversibleAction>) -> Result<(), ()> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = BroadcastActionsClient::new(client);

    let result = grpc
        .broadcast_actions(BroadcastActionsRequest {
            actions: map_vec(actions),
        })
        .await;

    result.map(|_| ()).map_err(|_| ())
}
