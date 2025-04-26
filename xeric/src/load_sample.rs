use itertools::Itertools;
use log::info;
use shared::load_sample::load_sample_server::LoadSample;
use shared::load_sample::{
    LoadSampleReply, LoadSampleRequest, LoadSampleTreeReply, LoadSampleTreeRequest,
};
use shared::model::{FileTreeConfig, FilenameTree, Sample};
use std::collections::HashSet;
use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{ReadDir, read_dir};
use std::io::Error;
use std::path::{Path, PathBuf};
use tonic::async_trait;

const _PCM_MAX_I16: i16 = 0x7FFF; // 2^15 - 1
const PCM_MAX_I24: i32 = 0x7FFFFF; // 2^23 - 1
const _PCM_MAX_I32: i32 = 0x7FFFFFFF; // 2^31 - 1
const _PCM_DIV_I16: f32 = 1.0 / _PCM_MAX_I16 as f32;
const PCM_DIV_I24: f32 = 1.0 / PCM_MAX_I24 as f32;
const _PCM_DIV_I32: f32 = 1.0 / _PCM_MAX_I32 as f32;

#[inline(always)]
pub fn to_f32(sample: i32) -> f32 {
    // I don't know where 128.0 comes from (other than that it's 2^7).
    // Perhaps the sample I was testing with (89 BPM F# Minor.wav) is actually 24 bit audio?
    PCM_DIV_I24 * sample as f32
}

// This is a stateless RPC, at least as far as in-memory state is concerned (it does
// depend on filesystem state). Therefore the context can be empty.
pub struct LoadSampleContext;

pub fn sample_dir_path() -> PathBuf {
    let mut file_path = current_dir().unwrap();
    file_path.pop(); // pop '/xeric'
    file_path.push("assets");
    file_path.push("samples");
    file_path
}

fn audio_file_types() -> HashSet<&'static str> {
    let mut audio_file_types = HashSet::new();
    audio_file_types.insert("wav");
    audio_file_types.insert("mp3");
    audio_file_types
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn dir_is_empty(dir: &FilenameTree) -> bool {
    match dir {
        FilenameTree::Directory(_, children) => children.is_empty(),
        FilenameTree::File(_) => true,
    }
}

// TODO: also search within directory names.
// If a directory matches the search, then all files within should display (except hidden/non-audio
// as appropriate).
fn read_dir_as_tree(path: PathBuf, config: &FileTreeConfig) -> Result<FilenameTree, Error> {
    let dir_contents: ReadDir = read_dir(&path)?;
    let filename = path
        .as_path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut tree = FilenameTree::Directory(filename, vec![]);
    let FilenameTree::Directory(_, children) = &mut tree else {
        panic!()
    };
    let audio_ext = audio_file_types();

    for file in dir_contents {
        let file = file?;
        let filename: String = file.file_name().to_str().unwrap().to_string();
        if file.metadata()?.is_dir() {
            if !config.show_hidden
                && (filename.starts_with(".") || filename.starts_with("__MACOSX"))
            {
                continue;
            }
            let dir = read_dir_as_tree(file.path(), config)?;
            if !dir_is_empty(&dir) {
                children.push(dir);
            }
        } else {
            let ext = get_extension_from_filename(&filename);
            if !config.search.is_empty()
                && !filename
                    .to_lowercase()
                    .contains(&config.search.to_lowercase())
            {
                continue;
            }
            if !config.show_non_audio && (ext.is_none() || !audio_ext.contains(ext.unwrap())) {
                continue;
            }
            if !config.show_hidden && filename.starts_with(".") {
                continue;
            }
            children.push(FilenameTree::File(filename));
        }
    }

    Ok(tree)
}

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

        let mut file_path = sample_dir_path();
        file_path.push(filename.clone());
        info!("Loading sample from path: {}", file_path.clone().display());

        // TODO: reading this seems to load at half the speed.
        // Perhaps the sample rate needs to be adjusted?
        // Either way, a bit weird and should be fixed.
        let mut reader = hound::WavReader::open(file_path).map_err(|_| {
            tonic::Status::invalid_argument(format!("File {} could not be read.", filename))
        })?;
        let chunks = reader
            .samples::<i32>()
            .chunks(2);
        let (left, right) = chunks
            .into_iter()
            .map(|mut data| {
                let left = to_f32(data.next().unwrap().unwrap());
                let right = to_f32(data.next().unwrap().unwrap());
                (left, right)
            })
            .unzip();
        let sample = Sample {
            left,
            right,
            sample_rate: reader.spec().sample_rate as f32,
        };
        Ok(tonic::Response::new(LoadSampleReply {
            sample: Some(sample.into()),
        }))
    }

    async fn load_sample_tree(
        self: &Self,
        request: tonic::Request<LoadSampleTreeRequest>,
    ) -> Result<tonic::Response<LoadSampleTreeReply>, tonic::Status> {
        let LoadSampleTreeRequest { config } = request.into_inner();
        let config: FileTreeConfig = config.unwrap().into();

        let tree = read_dir_as_tree(sample_dir_path(), &config)?;

        Ok(tonic::Response::new(LoadSampleTreeReply {
            tree: Some(tree.into()),
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
        let load_reply = my_load_sample.load_sample(load_request).await;

        // ASSERT
        // TODO more meaningful check of return.
        assert!(load_reply.is_ok())
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
        let load_reply = my_load_sample.load_sample(load_request).await;

        // ASSERT
        // TODO more meaningful check of return.
        assert!(load_reply.is_err())
    }
}
