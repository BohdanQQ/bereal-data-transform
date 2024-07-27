mod exporter_v0;

use std::path::{Path, PathBuf};

use chrono_tz::Tz;

pub const EXPORTER_COUNT: u64 = 1;

#[derive(Debug, Clone)]
pub struct BerealRecord {
    pub front_camera_path: PathBuf,
    pub back_camera_path: PathBuf,

    pub caption: String,
    pub naive_time_taken: chrono::NaiveDateTime,

    pub late: bool,

    pub song: Option<BerealSongData>,

    pub behind_the_scenes: Option<BerealBTSData>,
}

#[derive(Debug, Clone)]
pub enum BerealSongData {
    Spotify { spotify_song_id: String },
}

#[derive(Debug, Clone)]
pub enum BerealBTSData {
    Video { path: String },
}

pub trait Exporter {
    fn get_timezone(&self) -> Result<Tz, String>;
    fn parse_image_data(&self) -> Result<Vec<BerealRecord>, String>;
    fn check_file_structure(&self) -> Result<(), String>;
}

fn read_file_into_string<P>(path: P) -> Result<String, String>
where
    P: AsRef<Path> + Clone,
{
    let cloned = path.clone();
    let f = std::fs::File::open(path);
    if let Err(e) = f {
        return Err(format!(
            "Cannot open file {} - {}",
            cloned.as_ref().to_string_lossy(),
            e
        ));
    }

    let mut result: String = "".to_string();
    if let Err(e) = std::io::Read::read_to_string(&mut f.unwrap(), &mut result) {
        return Err(format!(
            "Read error, file {} - {}",
            cloned.as_ref().to_string_lossy(),
            e
        ));
    }
    Ok(result)
}

pub fn get_exporter(version: u64, input_path: &Path) -> Box<dyn Exporter> {
    if version >= EXPORTER_COUNT {
        panic!(
            "Export version invalid! Expecting number between 0 and {}",
            EXPORTER_COUNT - 1
        );
    }

    match version {
        0 => Box::new(exporter_v0::ExporterV0::new(PathBuf::from(input_path))),
        _ => panic!("Sanity check failed"),
    }
}
