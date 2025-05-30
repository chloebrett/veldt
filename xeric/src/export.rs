use chrono::{Datelike, Local};
use dasp_frame::Frame;
use hound::{SampleFormat, WavSpec, WavWriter};
use mesic::SAMPLE_RATE;
use mesic::graph::RenderGraph;
use mp3lame_encoder::{Builder, DualPcm, FlushNoGap, Id3Tag};
use shared::export::{ExportReply, ExportRequest, export_server::Export};
use shared::model::Project;
use state::StoreData;
use std::env::current_dir;
use std::fs::{File, create_dir_all};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use tonic::{Request, Response, Status, async_trait};

// Exports project to .wav
pub struct ExportContext;

// Created so that we can combine the file_path and dir_path functions.
enum AudioFileType {
    Mp3,
    Wav,
}

impl AudioFileType {
    fn as_str(&self) -> &'static str {
        match self {
            AudioFileType::Mp3 => "mp3",
            AudioFileType::Wav => "wav",
        }
    }
}

// Create output file path according to AudioFileType.
fn wav_file_path(name: &str, file_type: AudioFileType) -> PathBuf {
    let file_ext = file_type.as_str();
    let mut file_path = wav_dir_path(file_type);
    file_path.push(format!("{name}.{file_ext}"));
    file_path
}

fn wav_dir_path(file_type: AudioFileType) -> PathBuf {
    let mut dir_path = current_dir().unwrap();
    // Note: no need to pop '/xeric', as we assume we are running from the veldt dir.
    dir_path.push(file_type.as_str());
    dir_path
}

fn float_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

#[async_trait]
impl Export for ExportContext {
    async fn export(
        &self,
        request: Request<ExportRequest>,
    ) -> Result<Response<ExportReply>, Status> {
        let req = request.into_inner();
        let project: Project = req
            .project
            .ok_or(Status::invalid_argument("Project must be supplied"))?
            .into();

        // TODO: use the StoreData from the collab context.
        let store = StoreData {
            project: project.clone(),
            ..StoreData::default()
        };

        let graph = RenderGraph::without_rx(&store);

        let spec = WavSpec {
            channels: 2, // stereo
            sample_rate: SAMPLE_RATE as u32,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut buffer = Cursor::new(Vec::new());
        {
            let mut writer = WavWriter::new(&mut buffer, spec)
                .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;

            for frame in graph {
                for channel in 0..2 {
                    let sample = *frame.channel(channel).unwrap();

                    let sample_i16 =
                        (sample * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

                    writer
                        .write_sample(sample_i16)
                        .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;
                }
            }

            writer
                .finalize()
                .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;
        }

        let wav_bytes = buffer.into_inner();

        // write to output file
        let dir_path = wav_dir_path(AudioFileType::Wav);
        let _ = create_dir_all(&dir_path);
        let file_path = wav_file_path(&project.name, AudioFileType::Wav);

        // NOTE: will overwrite if the file already exists
        let mut out_file =
            File::create(&file_path).map_err(|e| tonic::Status::internal(format!("{e}")))?;
        out_file
            .write_all(&wav_bytes)
            .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;

        Ok(tonic::Response::new(ExportReply { audio: wav_bytes }))
    }

    // Code written using example from: https://docs.rs/mp3lame-encoder/latest/mp3lame_encoder/
    async fn export_mp3(
        &self,
        request: Request<ExportRequest>,
    ) -> Result<Response<ExportReply>, Status> {
        let req = request.into_inner();
        let project: Project = req
            .project
            .ok_or(Status::invalid_argument("Project must be supplied"))?
            .into();

        // TODO: use the StoreData from the collab context.
        let store = StoreData {
            project: project.clone(),
            ..StoreData::default()
        };

        let graph = RenderGraph::without_rx(&store);

        let (_, curr_year) = Local::now().year_ce();

        let mut mp3_encoder = Builder::new().expect("Create LAME builder");
        mp3_encoder.set_num_channels(2).expect("set channels");
        mp3_encoder
            .set_sample_rate(SAMPLE_RATE as u32)
            .expect("set sample rate");
        mp3_encoder
            .set_brate(mp3lame_encoder::Bitrate::Kbps320)
            .expect("set brate");
        mp3_encoder
            .set_quality(mp3lame_encoder::Quality::Best)
            .expect("set quality");
        mp3_encoder
            .set_id3_tag(Id3Tag {
                title: project.name.as_bytes(),
                artist: &[],
                album: &[],
                album_art: &[],
                year: curr_year.to_string().as_bytes(),
                comment: &[],
            })
            .expect("set id3 tags");

        let mut mp3_encoder = mp3_encoder.build().expect("Initialise LAME encoder");

        // Sample buffers.
        let mut left_channel = Vec::new();
        let mut right_channel = Vec::new();

        for frame in graph {
            left_channel.push(float_to_i16(frame[0]));
            right_channel.push(float_to_i16(frame[1]));
        }

        let input = DualPcm {
            left: &left_channel,
            right: &right_channel,
        };

        // There are some unsafe code executions here, but shouldn't be an issue so long as length of left and right channel are equal.
        // This was the solution provided with the docs, so I am unsure of if there is a better way.
        let mut mp3_out_buffer = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(input.left.len()));
        let encoded_size = mp3_encoder
            .encode(input, mp3_out_buffer.spare_capacity_mut())
            .expect("To encode");
        unsafe {
            mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
        }

        let encoded_size = mp3_encoder
            .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
            .expect("to flush");
        unsafe {
            mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
        }

        // write to output file
        let dir_path = wav_dir_path(AudioFileType::Mp3);
        let _ = create_dir_all(&dir_path);
        let file_path = wav_file_path(&project.name, AudioFileType::Mp3);

        // NOTE: will overwrite if the file already exists
        let mut out_file =
            File::create(&file_path).map_err(|e| tonic::Status::internal(format!("{e}")))?;
        out_file
            .write_all(&mp3_out_buffer)
            .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;

        Ok(tonic::Response::new(ExportReply {
            audio: mp3_out_buffer,
        }))
    }
}
