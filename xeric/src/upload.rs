use crate::load_sample::sample_dir_path;
use log::info;
use shared::upload::{UploadSampleReply, UploadSampleRequest, upload_server::Upload};
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use tonic::async_trait;

pub struct UploadContext;

#[async_trait]
impl Upload for UploadContext {
    async fn upload_sample(
        self: &Self,
        request: tonic::Request<UploadSampleRequest>,
    ) -> Result<tonic::Response<UploadSampleReply>, tonic::Status> {
        //extract data
        let UploadSampleRequest {
            name,
            uploaded_sample,
        } = request.into_inner();

        //format file path for where to save sample
        let mut file_path = sample_dir_path();
        file_path.push(name.clone());

        //create and write data
        let mut file = File::create(file_path)?;
        file.write_all(&uploaded_sample)?;

        info!("Saved {} to server.", name.clone());
        Ok(tonic::Response::new(UploadSampleReply {}))
    }
}
