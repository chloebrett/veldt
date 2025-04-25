use log::info;
use tonic::async_trait;

use shared::upload::{UploadSampleReply, UploadSampleRequest, upload_server::Upload};

use std::env::current_dir;
use std::fs::File;
use std::io::Write;

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
        let mut file_path = current_dir().unwrap();
        file_path.pop();
        file_path.push("assets");
        file_path.push("samples");
        file_path.push(name.clone());

        //creat and write data
        let mut file = File::create(file_path)?;
        file.write_all(&uploaded_sample)?;

        info!("Saved {} to server.", name.clone());
        Ok(tonic::Response::new(UploadSampleReply {}))
    }
}
