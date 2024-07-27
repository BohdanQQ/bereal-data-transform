mod args;
mod export;
mod filter;
mod group;
pub mod parser;

use std::path::PathBuf;

use args::Args;
use clap::Parser;
use export::*;
use filter::*;
use group::*;
use parser::*;

fn main() {
    let args = Args::parse();
    process(args).unwrap();
}

fn process(args: Args) -> Result<(), String> {
    match args.command {
        args::Commands::Memories {
            image_format,
            group,
            caption,
            interval,
        } => {
            let input_path = PathBuf::from(args.input);
            let exporter = get_parser(args.export_version, &input_path);
            exporter.check_file_structure()?;
            // TODO: use this either in the image pasing phase or in the filtering phase
            // (timestamps are in UTC)
            let tx = exporter.get_timezone();

            let data = exporter.parse_image_data()?;

            let mut data = filter_moments(data, caption, interval)?;

            let grouped_moments = group_moments(&mut data, group)?;

            export_moments(
                &grouped_moments,
                input_path,
                PathBuf::from(&args.output),
                image_format,
            );

            // TODO: inject metadata to the targets

            for d in grouped_moments {
                println!("{:?}", d);
            }
            todo!()
        }
        args::Commands::RealMojis {} => todo!("unsupported realmojis"),
    }
}
