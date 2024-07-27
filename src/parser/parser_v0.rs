use chrono::NaiveDateTime;
use chrono_tz::Tz;
use regex::{Captures, Regex, Replacer};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::{
    fmt,
    path::{absolute, PathBuf},
};

use super::{BerealBTSData, BerealExportParser, BerealMomentRecord, BerealSongData};

pub struct ParserV0 {
    input_path: PathBuf,
}

impl ParserV0 {
    const MEMORIES_FILE: &'static str = "memories.json";
    const USER_FILE: &'static str = "user.json";

    fn relative_path(&self, p: &str) -> PathBuf {
        self.input_path.join(p)
    }

    pub fn new(input_path: PathBuf) -> ParserV0 {
        ParserV0 { input_path }
    }
}

impl BerealExportParser for ParserV0 {
    fn get_timezone(&self) -> Result<Tz, String> {
        let read_res = super::read_file_into_string(self.relative_path(ParserV0::USER_FILE))?;

        let u_json = serde_json::from_str::<UserJson>(&read_res)
            .map_err(|e| format!("Error parsing user.json file: {}", e))?;

        println!(
            "Info: username = {}, timezone = {}",
            u_json.username, u_json.timezone,
        );
        if let Ok(path) = absolute(self.relative_path(&u_json.profile_picture.path)) {
            println!("profile picture path: {}", path.to_string_lossy());
        }

        let tz: Tz = u_json.timezone.parse().unwrap();

        Ok(tz)
    }

    fn check_file_structure(&self) -> Result<(), String> {
        let required_files = vec![self.relative_path(ParserV0::MEMORIES_FILE)];

        let warn_if_missing_files = vec![self.relative_path(ParserV0::USER_FILE)];

        for required in required_files {
            if !required.exists() {
                return Err(format!(
                    "Required file {} does not exist",
                    required.to_string_lossy()
                ));
            } else if !required.is_file() {
                return Err(format!(
                    "{} is not a file, file expected",
                    required.to_string_lossy()
                ));
            }
        }

        for warn_f in warn_if_missing_files {
            if !warn_f.exists() {
                println!(
                    "Warning: non-vital file {} does not exist",
                    warn_f.to_string_lossy()
                )
            }
        }

        Ok(())
    }
    fn parse_image_data(&self) -> Result<Vec<BerealMomentRecord>, String> {
        let read_res = super::read_file_into_string(self.relative_path(ParserV0::MEMORIES_FILE))?;
        let parsed: Vec<MemoryItemJson> = serde_json::from_str(&read_res)
            .map_err(|e| format!("Failed to parse memories: {}", e))?;

        let pre_result: Vec<Result<BerealMomentRecord, String>> =
            parsed.iter().map(|x| x.clone().try_into()).collect();
        let mut result = vec![];
        let mut errors = false;

        for r in pre_result {
            if let Err(e) = r {
                println!("Error when parsing a memory entry: {}", e);
                errors = true;
            } else {
                result.push(r.unwrap());
            }
        }

        if errors {
            println!("Errors present, check output");
        }

        Ok(result)
    }
}

#[derive(Deserialize)]
struct UserJson {
    timezone: String,
    username: String,
    #[serde(alias = "profilePicture")]
    profile_picture: ProfilePictureJson,
}

#[derive(Deserialize, Clone)]
struct ProfilePictureJson {
    path: String,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "provider")]
enum Music {
    #[serde(alias = "spotify")]
    Spotify {
        #[serde(alias = "providerId")]
        provider_id: String,
    },
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

#[derive(Deserialize, Clone)]
#[serde(tag = "mediaType")]
enum Media {
    #[serde(alias = "image")]
    Image(MediaInfo),
    #[serde(alias = "video")]
    Video(MediaInfo),
}

#[derive(Deserialize, Clone)]
struct MediaInfo {
    path: String,
}

#[derive(Deserialize, Clone)]
struct MemoryItemJson {
    #[serde(alias = "frontImage")]
    pub front_image: Media,
    #[serde(alias = "backImage")]
    pub back_image: Media,
    #[serde(alias = "btsMedia")]
    pub bts_media: Option<Media>,
    #[serde(alias = "isLate")]
    pub late: bool,
    pub music: Option<Music>,
    pub caption: Option<String>,
    #[serde(alias = "takenTime")]
    pub time_taken: NaiveTimeWrap,
}

#[derive(Clone)]
struct NaiveTimeWrap {
    time: NaiveDateTime,
}

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = NaiveTimeWrap;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A string representing a date time, e.g. 2024-07-22T09:11:05.339Z")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let time = NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S%.3fZ")
            .map_err(|e| E::custom(format!("invalid format: {}\nError: {}", v, e)))?;

        Ok(NaiveTimeWrap { time })
    }
}

impl<'de> Deserialize<'de> for NaiveTimeWrap {
    fn deserialize<D>(deserializer: D) -> Result<NaiveTimeWrap, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeVisitor)
    }
}

struct StartEndEraseRest;
impl Replacer for StartEndEraseRest {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(&caps["start"]);
        dst.push_str(&caps["end"]);
    }
}

fn strip_bereal_id_from_path(path: &str) -> Option<String> {
    if let Ok(regex) = Regex::new(r"^(?<start>\/Photos\/)(?<berealId>[a-zA-Z0-9]+\/)(?<end>.*)$") {
        if !regex.is_match(path) {
            return None;
        }
        return Some("./".to_string() + regex.replace(path, StartEndEraseRest).as_ref());
    }
    None
}

impl TryInto<BerealMomentRecord> for MemoryItemJson {
    type Error = String;

    fn try_into(self) -> Result<BerealMomentRecord, Self::Error> {
        // strips bereal id path, handles errors
        let transformer = |p: MediaInfo| {
            strip_bereal_id_from_path(&p.path).map_or_else(
                || Err("BeReal path not recognised!".to_string()),
                |v| Ok(PathBuf::from(v)),
            )
        };

        let back_path = match self.back_image {
            Media::Image(p) => transformer(p),
            _ => Err("Invalid media type for back image!".to_string()),
        }?;

        let front_path = match self.front_image {
            Media::Image(p) => transformer(p),
            _ => Err("Invalid media type for front image!".to_string()),
        }?;

        let song: Option<BerealSongData> = self.music.and_then(|v| {
            let result: Result<BerealSongData, String> = v.try_into();
            if let Err(e) = result {
                println!("failed to parse music part: {}", e);
                return None;
            }
            Some(result.unwrap())
        });

        Ok(BerealMomentRecord {
            back_camera_path: back_path,
            front_camera_path: front_path,
            caption: self.caption.unwrap_or("".to_string()),
            naive_time_taken: self.time_taken.time,
            late: self.late,
            song,
            behind_the_scenes: self.bts_media.and_then(|bts| match bts {
                Media::Image(_) => None,
                Media::Video(v) => match strip_bereal_id_from_path(&v.path) {
                    None => {
                        println!("Error pasing BTS path");
                        None
                    }
                    Some(p) => Some(BerealBTSData::Video {
                        path: PathBuf::from("./".to_string() + &p),
                    }),
                },
            }),
        })
    }
}

impl TryInto<BerealSongData> for Music {
    type Error = String;

    fn try_into(self) -> Result<BerealSongData, Self::Error> {
        match self {
            Self::Spotify { provider_id } => Ok(BerealSongData::Spotify {
                spotify_song_id: provider_id,
            }),
            Self::Unknown(x) => Err(format!("Unknown song provider: {}", x)),
        }
    }
}
