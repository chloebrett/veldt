use crate::load_sample::sample_dir_path;
use log::info;
use shared::upload::{UploadChunkRequest, UploadSampleReply, upload_server::Upload};
use std::process::Command;
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
        let mut file_name = String::new();

        // Loop over chunks
        while let Some(chunk) = stream.message().await? {
            // Check if this is the first chunk, if so initialise.
            if file.is_none() {
                file_name = chunk.file_name.clone();
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

        // Check for the mka file created by the mic input, convert it to wav.
        if !file_name.is_empty() && file_name.ends_with(".mka") {
            let input_path = {
                let mut p = sample_dir_path();
                p.push(file_name);
                p
            };

            let output_path = input_path.with_extension("wav");

            let status = Command::new("ffmpeg")
                .arg("-i")
                .arg(&input_path)
                .arg("-ar")
                .arg("44100") // Need to downsample from 48k.
                .arg(&output_path)
                .arg("-y") // Overwrite output file.
                .status()
                .map_err(|e| tonic::Status::internal(format!("Failed to run ffmpeg: {}", e)))?;

            if !status.success() {
                return Err(tonic::Status::internal("FFmpeg conversion failed"));
            }

            // Remove the original mka file.
            let _ = tokio::fs::remove_file(&input_path).await;
        }

        Ok(Response::new(UploadSampleReply { success: true }))
    }
}
