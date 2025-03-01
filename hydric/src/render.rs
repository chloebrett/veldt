use leptos::logging::log;
use mesic::create_demo_track;
use mesic::io::as_floats;
use shared::model::demo_option::DemoOption;
use shared::model::wave_type::WaveType;
use shared::render::{RenderReply, RenderRequest, render_client::RenderClient};
use shared::types::Beats;
use shared::types::*;
use shared::model::track::Track;
use tonic_web_wasm_client::Client;
use mesic::create_track;

pub async fn render(
    demo_option: DemoOption,
    custom_track_notes: Vec<i32>,
    wave: WaveType,
    bpm: Beats,
    volume: Volume,
    transpose_semitones: i32
) -> Vec<f32> {
    let base_url = "http://127.0.0.1:3000".to_string();
    let wasm_client = Client::new(base_url);
    let mut grpc = RenderClient::new(wasm_client);

    let track = match demo_option {
        DemoOption::Custom => create_track(custom_track_notes, wave, bpm, volume, transpose_semitones),
        _ => create_demo_track(demo_option, wave, bpm, volume, transpose_semitones),
    };

    let result = grpc
        .render(RenderRequest {
            track: Some(track.into()),
        })
        .await;

    // TODO: proper error handling
    let audio = result.map(|it| it.into_inner().audio).unwrap_or(vec![]);

    log!("{:?}", audio);

    // TODO: consider if we should just send the Vec<f32> directly over the wire
    // instead of serializing to bytes first?
    as_floats(&audio)
}
