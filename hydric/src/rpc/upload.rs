use shared::consts::XERIC_URL;
use shared::upload::UploadChunkRequest;
use shared::upload::upload_client::UploadClient;
use tonic_web_wasm_client::Client;

pub async fn upload_sample(file_name: String, bytes: Vec<u8>) -> Result<(), String> {
    // Streams chunks of data now.

    let client = Client::new(XERIC_URL.to_string());
    let mut grpc = UploadClient::new(client);

    /*
    Chunk size in MB, I believe the error point is > 4MB.
    Increasing this beyond 4 will create a network error, likely as result of tonic or grpc.
    */
    const CHUNK_SIZE: usize = 3 * 1024 * 1024;

    // ::iter borrows from the inputted bytes, so we need to take ownership.
    let owned_chunks: Vec<Vec<u8>> = bytes
        .chunks(CHUNK_SIZE)
        .map(|chunk| chunk.to_vec())
        .collect();

    let total_chunks = owned_chunks.len() as u32;

    // Create stream.
    let stream =
        futures::stream::iter(owned_chunks.into_iter().enumerate().map(move |(i, chunk)| {
            UploadChunkRequest {
                file_name: file_name.clone(),
                chunk_data: chunk,
                chunk_index: i as u32,
                total_chunks,
            }
        }));

    grpc.upload_sample(stream)
        .await
        .map(|_| ()) //Currently discarding response.
        .map_err(|e| format!("Upload failed: {:?}", e))
}
