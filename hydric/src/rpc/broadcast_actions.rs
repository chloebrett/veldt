use shared::broadcast_actions::{
    BroadcastActionsRequest, broadcast_actions_client::BroadcastActionsClient,
};
use shared::consts::XERIC_URL;
use shared::serialize::map_vec;
use state::ReversibleAction;
use tonic_web_wasm_client::Client;

pub async fn broadcast_actions(actions: Vec<ReversibleAction>) -> Result<(), tonic::Status> {
    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = BroadcastActionsClient::new(client);

    let result = grpc
        .broadcast_actions(BroadcastActionsRequest {
            actions: map_vec(actions),
        })
        .await;

    result.map(|_| ())
}
