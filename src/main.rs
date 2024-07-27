mod args;
pub mod exporter;
mod filter;

use std::path::PathBuf;

use args::Args;
use clap::Parser;
use exporter::get_exporter;
use filter::filter_photos;

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
            let exporter = get_exporter(args.export_version, &PathBuf::from(args.input));
            exporter.check_file_structure()?;
            // TODO: use this either in the image pasing phase or in the filtering phase
            // (photo timestamps are in UTC)
            let tx = exporter.get_timezone();
            

            let data = exporter.parse_image_data()?;

            let data = filter_photos(data, caption, interval)?;
            
            // todo export data to the output folder

            for d in data {
                println!("{:?}", d);
            }
            todo!()
        }
        args::Commands::RealMojis {} => todo!("unsupported realmojis"),
    }
}
