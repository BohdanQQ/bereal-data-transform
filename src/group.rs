use std::path::PathBuf;

use chrono::{Datelike, Timelike};
use itertools::Itertools;

use crate::{args::Grouping, parser::BerealMomentRecord};

/// represents the paths where
/// outputs will be saved, relative to the output directory
#[derive(Debug)]
pub struct OutputMomentSpec<'a> {
    pub folder: PathBuf,
    pub file_name_prefix: String,
    pub moment: &'a BerealMomentRecord,
}

fn translate_bereal_moment_to_name_prefix(moment: &BerealMomentRecord) -> String {
    let time = moment.naive_time_taken;
    format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        time.year(),
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
        time.second()
    )
}

pub fn group_moments(
    moments: &mut Vec<BerealMomentRecord>,
    grouping: Grouping,
) -> Result<Vec<OutputMomentSpec<'_>>, String> {
    let mut result = vec![];

    for moment in moments {
        let prefix = translate_bereal_moment_to_name_prefix(moment);

        result.push(OutputMomentSpec {
            file_name_prefix: prefix,
            folder: PathBuf::new(),
            moment,
        })
    }

    match grouping {
        Grouping::None => Ok(result),
        // I could be done in a single for loop conditionally nesting if needed
        // however whenever i use the group from (key, group), I move out of it
        // and grouping in another level (year->month) becomes impossible

        // I might be missing something but I think it should be possible to
        // re-evaluate the group

        // the way grouping works now is by appending a folder name after each group_* pass
        // in the Grouping::Day scnario, we first append year folder, then month folder
        // (creating month subfolders in year folders) and then day folders
        Grouping::Year => {
            group_year(&mut result);
            Ok(result)
        }
        Grouping::Month => {
            group_year(&mut result);
            group_month(&mut result);
            Ok(result)
        }
        Grouping::Day => {
            group_year(&mut result);
            group_month(&mut result);
            group_day(&mut result, false);
            Ok(result)
        }
        Grouping::DayFlat => {
            group_day(&mut result, true);
            Ok(result)
        }
    }
}

fn group_year(result: &mut [OutputMomentSpec<'_>]) {
    let year_group = result
        .iter_mut()
        .chunk_by(|v| v.moment.naive_time_taken.year());
    for (key, group) in &year_group {
        for moment in group {
            moment.folder = moment.folder.join(format!("{:04}", key));
        }
    }
}

fn group_month(result: &mut [OutputMomentSpec<'_>]) {
    let year_group = result
        .iter_mut()
        .chunk_by(|v| v.moment.naive_time_taken.year());
    for (_, group) in &year_group {
        let month_group = group.chunk_by(|v| v.moment.naive_time_taken.month());
        for (m_key, m_group) in &month_group {
            for moment in m_group {
                moment.folder = moment.folder.join(format!("{:02}", m_key));
            }
        }
    }
}

fn group_day(result: &mut [OutputMomentSpec<'_>], flat: bool) {
    let year_group = result
        .iter_mut()
        .chunk_by(|v| v.moment.naive_time_taken.year());
    for (y_key, group) in &year_group {
        let month_group = group.chunk_by(|v| v.moment.naive_time_taken.month());
        for (m_key, m_group) in &month_group {
            let day_group = m_group.chunk_by(|v| v.moment.naive_time_taken.day());
            for (d_key, d_group) in &day_group {
                for moment in d_group {
                    if flat {
                        moment.folder = moment
                            .folder
                            .join(format!("{:04}-{:02}-{:02}", y_key, m_key, d_key));
                    } else {
                        moment.folder = moment.folder.join(format!("{:02}", d_key));
                    }
                }
            }
        }
    }
}
