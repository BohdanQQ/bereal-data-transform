mod parser_v0;

use std::path::{Path, PathBuf};

use chrono_tz::Tz;

pub const PARSER_COUNT: u64 = 1;

#[derive(Debug, Clone)]
pub struct BerealMomentRecord {
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
    Video { path: PathBuf },
}

pub trait BerealMemoriesParser {
    fn get_timezone(&self) -> Result<Tz, String>;
    fn parse_memories(&self) -> Result<Vec<BerealMomentRecord>, String>;
    fn check_memories_files(&self) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct BerealRealmojiRecord {
    pub image_path: PathBuf,
    pub is_instant: bool,
    pub post_time: chrono::NaiveDateTime,
    pub emoji: String,
}

pub trait BerealRealmojiParser {
    fn parse_realmojis(&self) -> Result<Vec<BerealRealmojiRecord>, String>;
    fn check_realmoji_files(&self) -> Result<(), String>;
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

fn check_vesion_panic(version: u64) {
    if version >= PARSER_COUNT {
        panic!(
            "Export version invalid! Expecting number between 0 and {}",
            PARSER_COUNT - 1
        );
    }
}

pub fn get_memories_parser(version: u64, input_path: &Path) -> Box<dyn BerealMemoriesParser> {
    check_vesion_panic(version);

    match version {
        0 => Box::new(parser_v0::ParserV0::new(PathBuf::from(input_path))),
        _ => panic!("Sanity check failed"),
    }
}

pub fn get_realmojis_parser(version: u64, input_path: &Path) -> Box<dyn BerealRealmojiParser> {
    check_vesion_panic(version);

    match version {
        0 => Box::new(parser_v0::ParserV0::new(PathBuf::from(input_path))),
        _ => panic!("Sanity check failed"),
    }
}
