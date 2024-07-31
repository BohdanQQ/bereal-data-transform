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
    let input_path = PathBuf::from(args.input);
    let output_folder = PathBuf::from(&args.output);
    match args.command {
        args::Commands::Memories {
            image_format,
            group,
            caption,
            interval,
        } => {
            let parser = get_memories_parser(args.export_version, &input_path);
            parser.check_memories_files()?;
            // TODO: use this either in the image pasing phase or in the filtering phase
            // (timestamps are in UTC)
            let tx = parser.get_timezone();

            let data = parser.parse_memories()?;
            if args.verbose {
                println!("Total parsed moments: {}", data.len());
            }

            let mut data = filter_moments(data, caption, interval)?;
            let filtered = data.len();
            if args.verbose {
                println!("Filtered moments: {}", filtered);
            }

            let grouped_moments = group_moments(&mut data, group)?;
            if grouped_moments.len() != filtered {
                print!("Warning: grouping phase omitted data!");
            }
            if args.verbose {
                println!("Exporting");
            }

            let exported = export_generic(
                output_folder.clone(),
                ExportParameters {
                    input_path,
                    image_format,
                },
                &grouped_moments,
                args.verbose,
            );

            if args.verbose {
                println!("Fully exported {} out of {} moments", exported, grouped_moments.len());
            }

            Ok(())
        }
        args::Commands::Realmojis {
            group,
            image_format,
        } => {
            let parser = get_realmojis_parser(args.export_version, &input_path);
            parser.check_realmoji_files()?;
            let mojis = parser.parse_realmojis()?;

            let mojis = group_realmojis(&mojis, group)?;

            let exported = export_generic(
                output_folder.clone(),
                ExportParameters {
                    input_path,
                    image_format,
                },
                &mojis,
                args.verbose,
            );

            if args.verbose {
              println!("Fully exported {} out of {} realmojis", exported, mojis.len());
            }
            Ok(())
        }
    }
}
