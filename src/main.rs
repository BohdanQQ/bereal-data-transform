mod args;
mod export;
mod filter;
mod group;
pub mod parser;

use std::{path::PathBuf, sync::Arc, vec};

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
    let input_path = Arc::new(input_path);
    let output_folder = Arc::new(PathBuf::from(&args.output));
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

            let image_format = Arc::new(image_format);
            let exported = export_generic(
                &grouped_moments,
                move |moment| output_folder.join(moment.folder.clone()),
                move |x| {
                    let mut result = vec![
                        ExportJobSpec::ImageConvert {
                            output_file_name: x.file_name_prefix.clone() + "_camera_front",
                            original_image_path: input_path.join(&x.moment.front_camera_path),
                            output_format: image_format.as_ref().clone(),
                        },
                        ExportJobSpec::ImageConvert {
                            output_file_name: x.file_name_prefix.clone() + "_camera_back",
                            original_image_path: input_path.join(&x.moment.back_camera_path),
                            output_format: image_format.as_ref().clone(),
                        },
                    ];

                    if let Some(BerealBTSData::Video { path }) = &x.moment.behind_the_scenes {
                        result.push(ExportJobSpec::Copy {
                            output_file_name: x.file_name_prefix.clone() + "_BTS",
                            original_path: input_path.join(path),
                        });
                    }
                    result
                },
                args.verbose,
            );

            if args.verbose {
                println!("Exported {} moments", exported);
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

            let image_format = Arc::new(image_format);
            let exported = export_generic(
                &mojis,
                move |moment| output_folder.join(moment.folder.clone()),
                move |x| {
                    vec![ExportJobSpec::ImageConvert {
                        output_file_name: x.file_name_prefix.clone(),
                        original_image_path: input_path.join(&x.image_file),
                        output_format: image_format.as_ref().clone(),
                    }]
                },
                args.verbose,
            );

            if args.verbose {
                println!("Exported {} realmojis", exported);
            }
            Ok(())
        }
    }
}
