use crate::parser::EXPORTER_COUNT;
use chrono::{NaiveDate, NaiveDateTime, ParseResult};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Grouping {
    None,
    Year,
    Month,
    Day,
    DayFlat,
}

#[derive(Parser, Debug)]
#[command(version = "0.1")]
#[command(about = "BeReal data export tool")]
#[command(
    long_about = "BeReal export tool that converts, filters and groups images from a BeReal export. The tool also handles metadata."
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Unzipped BeReal data export folder path
    #[arg(short, long)]
    pub input: String,

    /// Output folder path
    #[arg(short, long)]
    pub output: String,

    #[arg(short, long, default_value_t = 0, value_parser = clap::value_parser!(u64).range(0..EXPORTER_COUNT))]
    /// Export structure version
    pub export_version: u64,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Export BeReal Memories
    Memories {
        /// Converts image to the specified format
        #[arg(short, long)]
        #[clap(value_enum, default_value_t=ImageFormat::Jpeg)]
        image_format: ImageFormat,

        /// Groups images and videos
        #[arg(short, long)]
        #[clap(value_enum, default_value_t=Grouping::None)]
        group: Grouping,

        /// Caption regular expression filter
        #[arg(short, long)]
        caption: Option<String>,

        /// Time filter list. Specify a pairs of values separated by comma. Each pair is separated the plus sign (+).
        /// Each value shall be in the form of either YYYY-MM-DDTHH:mm:SS or YYYY-MM-DD. Order within the list and pairs is irrelevant.
        /// (example: 2024-02-10+2022-01-19T13:51:00,2021-08-19T20:11:32+2021-09-20T20:11:32 will search within 2 time intervals)
        #[arg(short = 't', long, value_parser = parse_interval_vec)]
        interval: Option<std::vec::Vec<TimeInterval>>,
    },

    /// Export RealMojis
    RealMojis {
        //TODO!
    },
}

#[derive(Debug, Clone)]
pub struct TimeInterval {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
}

fn parse_interval_vec(arg: &str) -> Result<Vec<TimeInterval>, String> {
    arg.split(',')
        .filter(|i| !i.is_empty())
        .map(parse_interval)
        .try_fold(vec![], |mut acc, val| {
            val.map(|v| {
                acc.push(v);
                acc
            })
        })
}

fn parse_interval(arg: &str) -> Result<TimeInterval, String> {
    let split: Vec<&str> = arg.split('+').collect();
    if split.len() != 2 {
        return Err("Invalid format, expecting exactly two time points.".to_string());
    }

    let parsing_results: Vec<ParseResult<NaiveDateTime>> = split
        .into_iter()
        .map(|x| {
            // YYYY-MM-DDTHH:mm:SS or YYYY-MM-DD
            NaiveDateTime::parse_from_str(x, "%Y-%m-%dT%H:%M:%S")
                .or(NaiveDate::parse_from_str(x, "%Y-%m-%d").map(NaiveDateTime::from))
        })
        .collect();

    if parsing_results.iter().any(|res| res.is_err()) {
        return Err("One of the supplied time points are invalid".to_string());
    }

    let mut time_objs = parsing_results
        .into_iter()
        .map(|x| x.unwrap())
        .collect::<Vec<NaiveDateTime>>();

    time_objs.sort();
    if time_objs.len() != 2 {
        return Err("Sanity check failed".to_string());
    }
    Ok(TimeInterval {
        from: time_objs[0],
        to: time_objs[1],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_example_value() {
        let result = parse_interval_vec(
            "2024-02-10+2022-01-19T13:51:00,2021-08-19T20:11:32+2021-09-20T20:11:32",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2)
    }

    #[test]
    fn fail_on_pair_invalid_extra_1() {
        let result = parse_interval_vec("2024-02-10+2022-01-19T13:51:00+");
        assert!(result.is_err());
    }

    #[test]
    fn fail_on_pair_invalid_extra_2() {
        let result = parse_interval_vec("+2024-02-10+2022-01-19T13:51:00");
        assert!(result.is_err());
    }

    #[test]
    fn fail_on_pair_invalid_extra_3() {
        let result = parse_interval_vec("2024-02-10++2022-01-19T13:51:00");
        assert!(result.is_err());
    }

    #[test]
    fn fail_on_pair_invalid_extra_4() {
        let result = parse_interval_vec("++2024-02-10++2022-01-19T13:51:00+");
        assert!(result.is_err());
    }

    #[test]
    fn do_not_fail_on_list_dangling_pre() {
        let result = parse_interval_vec(",2024-02-10+2022-01-19T13:51:00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1)
    }

    #[test]
    fn do_not_fail_on_list_dangling_post() {
        let result = parse_interval_vec("2024-02-10+2022-01-19T13:51:00,");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1)
    }

    #[test]
    fn do_not_fail_on_list_dangling_mid() {
        let result = parse_interval_vec(
            "2024-02-10+2022-01-19T13:51:00,,2021-08-19T20:11:32+2021-09-20T20:11:32",
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2)
    }
}
