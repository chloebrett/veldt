use shared::load_sample::load_sample_server::LoadSample;
use shared::load_sample::{LoadSampleReply, LoadSampleRequest};
use tonic::async_trait;
use shared::model::Sample;

pub struct MyLoadSample;

#[async_trait]
impl LoadSample for MyLoadSample {
    async fn load_sample(
        self: &Self,
        request: tonic::Request<LoadSampleRequest>,
    ) -> Result<tonic::Response<LoadSampleReply>, tonic::Status> {
        let LoadSampleRequest { filename } = request.into_inner();
        let mut reader = hound::WavReader::open(filename).unwrap();
        let data: Vec<f32> = reader.samples::<f32>().into_iter().map(|it| it.unwrap()).collect();
        let sample = Sample {
            data,
            sample_rate: reader.spec().sample_rate as f32,
        };
        Ok(tonic::Response::new(LoadSampleReply { sample: Some(sample.into()) }))
    }
}
