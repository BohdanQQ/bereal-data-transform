use crate::{args::TimeInterval, parser::BerealMomentRecord};

pub fn filter_moments(
    moments: Vec<BerealMomentRecord>,
    caption_regex: Option<String>,
    intervals_allowed: Option<Vec<TimeInterval>>,
) -> Result<Vec<BerealMomentRecord>, String> {
    let mut result: Vec<BerealMomentRecord> = vec![];
    let mut regex = None;
    if let Some(rexp) = caption_regex {
        let re = regex::Regex::new(&rexp.to_lowercase());
        if let Err(e) = re {
            return Err(format!("invalid regex: {}", e));
        }

        regex = Some(re.unwrap());
    }

    let filter_present = regex.is_some() || intervals_allowed.is_some();
    if !filter_present {
        return Ok(moments);
    }

    for photo in moments {
        if let Some(regex) = regex.as_ref() {
            if !regex.is_match(&photo.caption.to_lowercase()) {
                continue;
            }
        }

        if let Some(times) = intervals_allowed.as_ref() {
            let mut matches_time = false;
            for intvl in times {
                if photo.naive_time_taken < intvl.from || photo.naive_time_taken > intvl.to {
                    continue;
                }
                matches_time = true;
                break;
            }

            if matches_time {
                result.push(photo.clone());
            }
        } else {
            result.push(photo.clone());
        }
    }

    Ok(result)
}
