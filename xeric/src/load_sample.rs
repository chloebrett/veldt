use log::info;
use shared::load_sample::load_sample_server::LoadSample;
use shared::load_sample::{LoadSampleReply, LoadSampleRequest};
use shared::model::Sample;
use std::env::current_dir;
use tonic::async_trait;

const PCM_MAX_I16: i16 = 0x7FFF; // 2^15 - 1
const _PCM_MAX_I32: i32 = 0x7FFFFFFF; // 2^31 - 1
const PCM_DIV_I16: f32 = 1.0 / PCM_MAX_I16 as f32;
const _PCM_DIV_I32: f32 = 1.0 / _PCM_MAX_I32 as f32;

#[inline(always)]
pub fn to_f32(sample: i32) -> f32 {
    // I don't know where 128.0 comes from (other than that it's 2^7).
    // Perhaps the sample I was testing with (89 BPM F# Minor.wav) is actually 24 bit audio?
    PCM_DIV_I16 * sample as f32 / 128.0
}

// This is a stateless RPC, at least as far as in-memory state is concerned (it does
// depend on filesystem state). Therefore the context can be empty.
pub struct LoadSampleContext;

#[async_trait]
impl LoadSample for LoadSampleContext {
    /// Loads a sample from the server filesystem by name.
    /// It's assumed that the file exists in veldt/assets/samples.
    /// Throws an error if the file does not exist. Panics if reading it fails.
    async fn load_sample(
        self: &Self,
        request: tonic::Request<LoadSampleRequest>,
    ) -> Result<tonic::Response<LoadSampleReply>, tonic::Status> {
        let LoadSampleRequest { filename } = request.into_inner();

        let mut file_path = current_dir().unwrap();
        file_path.pop(); // pop '/xeric'
        file_path.push("assets");
        file_path.push("samples");
        file_path.push(filename.clone());
        info!("Loading sample from path: {}", file_path.clone().display());

        // TODO: reading this seems to load at half the speed.
        // Perhaps the sample rate needs to be adjusted?
        // Either way, a bit weird and should be fixed.
        let mut reader = hound::WavReader::open(file_path).map_err(|_| {
            tonic::Status::invalid_argument(format!("File {} could not be read.", filename))
        })?;
        let data: Vec<f32> = reader
            .samples::<i32>()
            .map(|it| to_f32(it.unwrap()))
            .collect();
        let sample = Sample {
            data,
            sample_rate: reader.spec().sample_rate as f32,
        };
        Ok(tonic::Response::new(LoadSampleReply {
            sample: Some(sample.into()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_sample() {
        // ARRANGE
        let my_load_sample = LoadSampleContext;
        let sample_name = "89 BPM F# Minor.wav";
        let load_request = tonic::Request::new(LoadSampleRequest {
            filename: sample_name.into(),
        });

        // ACT
        let load_request = my_load_sample.load_sample(load_request).await;

        // ASSERT
        // TODO more meaningful check of return.
        assert!(load_request.is_ok())
    }

    #[tokio::test]
    async fn load_invalid_file_name_fails() {
        // ARRANGE
        let my_load_sample = LoadSampleContext;
        let sample_name = "test.wav";
        let load_request = tonic::Request::new(LoadSampleRequest {
            filename: sample_name.into(),
        });

        // ACT
        let load_request = my_load_sample.load_sample(load_request).await;

        // ASSERT
        // TODO more meaningful check of return.
        assert!(load_request.is_err())
    }
}
