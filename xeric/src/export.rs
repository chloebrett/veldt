use dasp_frame::Frame;
use hound::{SampleFormat, WavSpec, WavWriter};
use mesic::SAMPLE_RATE;
use mesic::render;
use shared::export::{ExportReply, ExportRequest, export_server::Export};
use std::env::current_dir;
use std::fs::{File, create_dir_all};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use tonic::async_trait;

// Exports project to .wav
pub struct ExportContext;

// Create output .wav file path
fn wav_file_path(name: &str) -> PathBuf {
    let mut file_path = wav_dir_path();
    file_path.push(format!("{name}.wav"));
    file_path
}

fn wav_dir_path() -> PathBuf {
    let mut dir_path = current_dir().unwrap();
    dir_path.pop(); // pop '/xeric'
    dir_path.push("wav");
    dir_path
}

#[async_trait]
impl Export for ExportContext {
    async fn export(
        &self,
        request: tonic::Request<ExportRequest>,
    ) -> Result<tonic::Response<ExportReply>, tonic::Status> {
        let req = request.into_inner();
        let project = req
            .project
            .ok_or_else(|| tonic::Status::invalid_argument("Project must be supplied"))?
            .into();

        let graph = &mut render(&project);

        let spec = WavSpec {
            channels: 1, // mono
            sample_rate: SAMPLE_RATE as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec)
                .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;

            for frame in graph {
                let sample = *frame.channel(0).unwrap(); // mono
                let sample_i16 =
                    (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

                writer
                    .write_sample(sample_i16)
                    .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
            }

            writer
                .finalize()
                .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;
        }

        let wav_bytes = buffer.into_inner();

        // write to output file
        let dir_path = wav_dir_path();
        let _ = create_dir_all(&dir_path);
        let file_path = wav_file_path(&project.name);

        // NOTE: will overwrite if the file already exists
        let mut out_file =
            File::create(&file_path).map_err(|e| tonic::Status::internal(format!("{}", e)))?;
        out_file
            .write_all(&wav_bytes)
            .map_err(|e| tonic::Status::invalid_argument(format!("{}", e)))?;

        Ok(tonic::Response::new(ExportReply { audio: wav_bytes }))
    }
}
