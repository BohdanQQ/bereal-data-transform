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
    let para_coeff = args.parallelism as f32 / 100.0;
    match args.command {
        args::Commands::Memories {
            image_format,
            group,
            caption,
            interval,
            desc_prefix,
            desc_suffix,
        } => {
            let parser = get_memories_parser(args.export_version, &input_path);
            parser.check_memories_files()?;
            // TODO: use this either in the image pasing phase or in the filtering phase
            // (timestamps are in UTC)
            let _tz = parser.get_timezone();

            let data = parser.parse_memories()?;
            if args.verbose {
                println!("Total parsed moments: {}", data.len());
            }

            let mut data = filter_moments(data, caption, interval.unwrap_or(vec![]))?;
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
                    desc_prefix,
                    desc_suffix,
                    disable_metadata: args.no_meta,
                },
                &grouped_moments,
                args.verbose,
                para_coeff,
            );

            if args.verbose {
                println!(
                    "Fully exported {} out of {} moments",
                    exported,
                    grouped_moments.len()
                );
            }

            Ok(())
        }
        args::Commands::Realmojis {
            group,
            image_format,
            desc_prefix,
            desc_suffix,
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
                    desc_prefix,
                    desc_suffix,
                    disable_metadata: args.no_meta,
                },
                &mojis,
                args.verbose,
                para_coeff,
            );

            if args.verbose {
                println!(
                    "Fully exported {} out of {} realmojis",
                    exported,
                    mojis.len()
                );
            }
            Ok(())
        }
    }
}
