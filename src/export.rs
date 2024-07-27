use crate::{args::ImageFormat, group, BerealBTSData};
use image::ImageReader;
use std::{
    fmt::Display,
    fs::{self, canonicalize, File},
    io::{self, BufWriter},
    path::{absolute, Path, PathBuf},
};

pub fn export_moments(
    moment_output_spec: &Vec<group::OutputMomentSpec>,
    input_folder: PathBuf,
    output_folder: PathBuf,
    format: ImageFormat,
) {
    let image_extension = match format {
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Png => "png",
    };
    let lib_format = match format {
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::Png => image::ImageFormat::Png,
    };

    // TODO: Paralelize
    for moment in moment_output_spec {
        let folder = output_folder.join(moment.folder.clone());
        if !folder.exists() {
            if let Err(e) = fs::create_dir_all(&folder) {
                println!("Failed to create directories {}\nSkipping...", e);
                continue;
            }
        }

        let front_name = moment.file_name_prefix.clone() + "_camera_front." + image_extension;
        let res_front = convert_to(
            &input_folder.join(&moment.moment.front_camera_path),
            &folder.join(front_name),
            lib_format,
        );
        print_if_err(&res_front);

        let back_name = moment.file_name_prefix.clone() + "_camera_ back." + image_extension;
        let res_back = convert_to(
            &input_folder.join(&moment.moment.back_camera_path),
            &folder.join(back_name),
            lib_format,
        );
        print_if_err(&res_back);

        if let Some(BerealBTSData::Video { path }) = &moment.moment.behind_the_scenes {
            let bts_name = moment.file_name_prefix.clone() + "_BTS";
            let mb_ext = Path::new(path).extension().and_then(|s| s.to_str());
            if let Some(ext) = mb_ext {
                let res_bts = copy_to(&input_folder.join(path), &folder.join(bts_name + "." + ext));
                print_if_err(&res_bts);
            }
        }
    }
}

fn convert_to(
    from: &PathBuf,
    to: &PathBuf,
    format: image::ImageFormat,
) -> Result<(), image::ImageError> {
    let file = File::create(absolute(to)?)?;
    let mut target = BufWriter::new(file);
    let img = ImageReader::open(canonicalize(from)?)?
        .with_guessed_format()?
        .decode()?;
    img.write_to(&mut target, format)
}

fn copy_to(from: &PathBuf, to: &PathBuf) -> Result<(), io::Error> {
    fs::copy(from, to)?;
    Ok(())
}

fn print_if_err<R, E: Display>(res: &Result<R, E>) {
    if let Err(e) = res {
        println!("{}", e);
    }
}
