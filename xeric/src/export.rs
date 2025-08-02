use chrono::{Datelike, Local};
use dasp_frame::Frame;
use hound::{SampleFormat, WavSpec, WavWriter};
use mesic::SAMPLE_RATE;
use mesic::consts::CHANNEL_COUNT;
use mesic::graph::RenderGraph;
use mp3lame_encoder::{Builder, DualPcm, FlushNoGap, Id3Tag};
use shared::export::{ExportReply, ExportRequest, export_server::Export};
use shared::model::Project;
use state::StoreData;
use std::env::current_dir;
use std::fs::{File, create_dir_all};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use strum::Display;
use tonic::{Request, Response, Status, async_trait};

// Exports project to .wav or .mp3
pub struct ExportContext;

#[derive(Display)]
enum AudioFileType {
    #[strum(to_string = "mp3")]
    Mp3,
    #[strum(to_string = "wav")]
    Wav,
}

// Create output file path according to AudioFileType.
fn export_file_path(name: &str, file_type: AudioFileType) -> PathBuf {
    let file_ext = file_type.to_string();
    let mut file_path = export_dir_path(file_type);
    file_path.push(format!("{name}.{file_ext}"));
    file_path
}

fn export_dir_path(file_type: AudioFileType) -> PathBuf {
    let mut dir_path = current_dir().unwrap();
    dir_path.push("assets");
    dir_path.push("exports");
    // Note: no need to pop '/xeric', as we assume we are running from the veldt dir.
    dir_path.push(file_type.to_string());
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
        let config = req
            .config
            .ok_or(Status::invalid_argument("Config must be supplied"))?;

        match config.audio_type.as_str() {
            "mp3" => export_mp3(&project),
            "wav" => export_wav(&project),
            _ => Err(Status::invalid_argument("Invalid audio type")),
        }
    }
}

// TODO: Work out how to either box the status cleanly, or implement custom error types for export_wav and export_mp3
// to resolve this linting warning. Currently tonic is quite specific on the result it wants, which causes issues when using 
// https://docs.rs/anyhow/latest/anyhow/ or a box. However, error is only 170 bytes, so not huge performance loss.
// Clickup bug bounty: https://app.clickup.com/t/86czvkxj9
#[allow(clippy::result_large_err)]
fn export_wav(project: &Project) -> Result<Response<ExportReply>, Status> {
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

    // Write to output file.
    let dir_path = export_dir_path(AudioFileType::Wav);
    let _ = create_dir_all(&dir_path);
    let file_path = export_file_path(&project.name, AudioFileType::Wav);

    // NOTE: will overwrite if the file already exists.
    let mut out_file =
        File::create(&file_path).map_err(|e| tonic::Status::internal(format!("{e}")))?;
    out_file
        .write_all(&wav_bytes)
        .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;

    Ok(tonic::Response::new(ExportReply { audio: wav_bytes }))
}

fn make_mp3_encoder(
    num_channels: u8,
    sample_rate: u32,
    bit_rate: mp3lame_encoder::Bitrate,
    quality: mp3lame_encoder::Quality,
    id3_tag: Id3Tag,
) -> mp3lame_encoder::Encoder {
    let mut mp3_encoder = Builder::new().expect("Create LAME builder");

    mp3_encoder
        .set_num_channels(num_channels)
        .expect("set channels");
    mp3_encoder
        .set_sample_rate(sample_rate)
        .expect("set sample rate");
    // TODO: Allow user to specify bitrate, common options are 320, 256, 192 and 128kbps.
    mp3_encoder.set_brate(bit_rate).expect("set brate");
    mp3_encoder.set_quality(quality).expect("set quality");
    mp3_encoder.set_id3_tag(id3_tag).expect("set id3 tags");

    mp3_encoder.build().expect("Initialise LAME encoder")
}

// Code written using example from: https://docs.rs/mp3lame-encoder/latest/mp3lame_encoder/
#[allow(clippy::result_large_err)]
fn export_mp3(project: &Project) -> Result<Response<ExportReply>, Status> {
    // TODO: use the StoreData from the collab context.
    let store = StoreData {
        project: project.clone(),
        ..StoreData::default()
    };

    let graph = RenderGraph::without_rx(&store);

    // Setup values for encoder builder.
    let (_, curr_year) = Local::now().year_ce();
    let curr_year = curr_year.to_string();
    let channel_count_u8 = CHANNEL_COUNT as u8;
    let sample_rate_u32 = SAMPLE_RATE as u32;
    let bit_rate = mp3lame_encoder::Bitrate::Kbps320;
    let quality = mp3lame_encoder::Quality::Best;
    let id3_tag = Id3Tag {
        title: project.name.as_bytes(),
        artist: &[],
        album: &[],
        album_art: &[],
        year: curr_year.as_bytes(),
        comment: &[],
    };

    let mut mp3_encoder = make_mp3_encoder(
        channel_count_u8,
        sample_rate_u32,
        bit_rate,
        quality,
        id3_tag,
    );

    // Sample buffers.
    let mut left_channel = vec![];
    let mut right_channel = vec![];

    // Note that docs specify u16, but this is incorrect.
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
    let mut mp3_out_buffer =
        Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(input.left.len()));
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

    // Write to output file.
    let dir_path = export_dir_path(AudioFileType::Mp3);
    let _ = create_dir_all(&dir_path);
    let file_path = export_file_path(&project.name, AudioFileType::Mp3);

    // NOTE: will overwrite if the file already exists.
    let mut out_file =
        File::create(&file_path).map_err(|e| tonic::Status::internal(format!("{e}")))?;
    out_file
        .write_all(&mp3_out_buffer)
        .map_err(|e| tonic::Status::invalid_argument(format!("{e}")))?;

    Ok(tonic::Response::new(ExportReply {
        audio: mp3_out_buffer,
    }))
}
