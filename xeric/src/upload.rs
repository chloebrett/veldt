use crate::load_sample::sample_dir_path;
use log::info;
use shared::upload::{UploadChunkRequest, UploadSampleReply, upload_server::Upload};
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming, async_trait};

pub struct UploadContext;

#[async_trait]
impl Upload for UploadContext {
    async fn upload_sample(
        self: &Self,
        request: Request<Streaming<UploadChunkRequest>>,
    ) -> Result<Response<UploadSampleReply>, Status> {
        let mut stream = request.into_inner();
        let mut file = None;

        // Loop over chunks
        while let Some(chunk) = stream.message().await? {
            // Check if this is the first chunk, if so initialise.
            if file.is_none() {
                let path = {
                    let mut p = sample_dir_path();
                    p.push(&chunk.file_name);
                    p
                };

                // Use tokio as it has better async handling of files.
                let f = tokio::fs::File::create(&path)
                    .await
                    .map_err(|e| tonic::Status::internal(format!("Failed to create file: {e}")))?;

                file = Some(f);
            }

            // Write to file
            file.as_mut()
                .unwrap()
                .write_all(&chunk.chunk_data)
                .await
                .map_err(|e| {
                    tonic::Status::internal(format!(
                        "Failed to write chunk {}: {}",
                        chunk.chunk_index, e
                    ))
                })?;

            info!(
                "Received chunk {}/{} for {}",
                chunk.chunk_index + 1,
                chunk.total_chunks,
                chunk.file_name
            );
        }

        Ok(Response::new(UploadSampleReply { success: true }))
    }
}
