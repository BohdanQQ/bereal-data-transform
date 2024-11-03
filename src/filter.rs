use itertools::Itertools;

use crate::{args::TimeInterval, parser::BerealMomentRecord};

pub fn filter_moments(
    moments: Vec<BerealMomentRecord>,
    caption_regex: Option<String>,
    intervals_allowed: Vec<TimeInterval>,
) -> Result<Vec<BerealMomentRecord>, String> {
    let mut result: Vec<BerealMomentRecord> = vec![];
    let mut regex = None;
    if let Some(rexp) = caption_regex {
        let re =
            regex::Regex::new(&rexp.to_lowercase()).map_err(|e| format!("invalid regex: {}", e))?;
        regex = Some(re);
    }

    let time_fillter_present = !intervals_allowed.is_empty();
    if regex.is_none() && !time_fillter_present {
        return Ok(moments);
    }

    for photo in moments {
        // "continue" in this loop means "photo did not pass filtering"
        if let Some(regex) = regex.as_ref() {
            if !regex.is_match(&photo.caption.to_lowercase()) {
                continue;
            }
        }

        let passed_one_or_zero = intervals_allowed
            .iter()
            .filter(|t| photo.naive_time_taken >= t.from && photo.naive_time_taken <= t.to)
            .take(1)
            .collect_vec()
            .len();

        if !time_fillter_present || passed_one_or_zero > 0 {
            result.push(photo.clone());
        }
    }

    Ok(result)
}
