use crate::{args::ImageFormat, group, BerealBTSData};
use image::ImageReader;
use std::{
    cmp::max,
    fmt::Display,
    fs::{self, canonicalize, File},
    io::{self, BufWriter, Write},
    path::{absolute, Path, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
    thread,
    time::Duration,
};

pub fn export_moments(
    moment_output_spec: &Vec<group::OutputMomentSpec>,
    input_folder: PathBuf,
    output_folder: PathBuf,
    format: ImageFormat,
    verbose: bool,
) -> usize {
    let total = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicUsize::new(0));
    let image_extension = match format {
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Png => "png",
    };
    let lib_format = match format {
        ImageFormat::Jpeg => image::ImageFormat::Jpeg,
        ImageFormat::Png => image::ImageFormat::Png,
    };
    if verbose {
        println!("Spawning filesystem structure");
    }
    for moment in moment_output_spec {
        let folder = output_folder.join(moment.folder.clone());
        if !folder.exists() {
            if let Err(e) = fs::create_dir_all(&folder) {
                println!("Failed to create directories {}\nSkipping...", e);
                continue;
            }
        }
    }

    let cpu_count = max(1, num_cpus::get());
    let chunk_size = (moment_output_spec.len() + cpu_count - 1) / cpu_count;
    let thread_count = moment_output_spec.chunks(chunk_size).len();
    if verbose {
        println!(
            "Converting... 1-T Workload: {}, Thread Count: {}",
            chunk_size, thread_count
        );
    }
    thread::scope(|s| {
        for chunk in moment_output_spec.chunks(chunk_size) {
            let output_folder = output_folder.clone();
            let input_folder = input_folder.clone();
            let total = Arc::clone(&total);
            let done = Arc::clone(&done);
            s.spawn(move || {
                for moment in chunk {
                    let folder = output_folder.join(moment.folder.clone());
                    if !folder.exists() {
                        println!("Directory not found {}\nSkipping...", folder.display());
                        continue;
                    }

                    let front_name =
                        moment.file_name_prefix.clone() + "_camera_front." + image_extension;
                    let res_front = convert_to(
                        &input_folder.join(&moment.moment.front_camera_path),
                        &folder.join(front_name),
                        lib_format,
                    );
                    print_if_err(&res_front);

                    let back_name =
                        moment.file_name_prefix.clone() + "_camera_ back." + image_extension;
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
                            let res_bts = fs::copy(
                                &input_folder.join(path),
                                &folder.join(bts_name + "." + ext),
                            );
                            print_if_err(&res_bts);
                        }
                    }
                    total.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                }
                done.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }

        let total = Arc::clone(&total);
        let done = Arc::clone(&done);
        s.spawn(move || {
            while done.load(std::sync::atomic::Ordering::SeqCst) != thread_count {
                if verbose {
                    print!(
                        "\rProgress: {}/{} ({} threads done)",
                        total.load(std::sync::atomic::Ordering::SeqCst),
                        moment_output_spec.len(),
                        done.load(std::sync::atomic::Ordering::SeqCst)
                    );
                }
                let _ = io::stdout().flush();
                thread::sleep(Duration::from_millis(500));
            }
            if verbose {
                println!(
                    "\rProgress: {}/{} ({} threads done)",
                    total.load(std::sync::atomic::Ordering::SeqCst),
                    moment_output_spec.len(),
                    done.load(std::sync::atomic::Ordering::SeqCst)
                );
            }
        });
    });

    total.load(std::sync::atomic::Ordering::SeqCst)
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

fn print_if_err<R, E: Display>(res: &Result<R, E>) {
    if let Err(e) = res {
        println!("{}", e);
    }
}
