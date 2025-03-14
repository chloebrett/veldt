use shared::load_sample::load_sample_server::LoadSample;
use shared::load_sample::{LoadSampleRequest, LoadSampleReply};
use shared::bytes::as_bytes;
use tonic::async_trait;

pub struct MyLoadSample;

#[async_trait]
impl LoadSample for MyLoadSample {
    async fn load_sample(
        self: &Self,
        _request: tonic::Request<LoadSampleRequest>,
    ) -> Result<tonic::Response<LoadSampleReply>, tonic::Status> {
        let bytes = as_bytes(&vec![]);
        Ok(tonic::Response::new(LoadSampleReply { sample: bytes }))
    }
}
