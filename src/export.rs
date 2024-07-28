use crate::args::ImageFormat;
use image::ImageReader;
use std::{
    cmp::max,
    fmt::Display,
    fs::{self, canonicalize, File},
    io::{self, BufWriter, Write},
    path::{absolute, PathBuf},
    sync::{atomic::AtomicUsize, Arc},
    thread,
    time::Duration,
};

pub enum ExportJobSpec {
    ImageConvert {
        /// filename WITHOUT the extension
        output_file_name: String,
        /// image to convert
        original_image_path: PathBuf,
        output_format: ImageFormat,
    },
    Copy {
        /// where to copy to, WITHOUT file extension (will be copied from the original)
        output_file_name: String,
        /// where to copy from
        original_path: PathBuf,
    },
}

/// # Arguments
/// * `path_generator` - generates path where the entry will be stored - just the folder
/// * `export_job_generator` - generates exports jobs that will execute withing the output folder supplied by the path_generator argument
pub fn export_generic<T, PathGen, JobGen>(
    moment_output_spec: &Vec<T>,
    path_generator: PathGen,
    export_job_generator: JobGen,
    verbose: bool,
) -> usize
where
    PathGen: Fn(&T) -> PathBuf + Send + Sync,
    JobGen: Fn(&T) -> Vec<ExportJobSpec> + Send + Sync,
    T: Send + Sync,
{
    let total = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicUsize::new(0));

    if verbose {
        println!("Spawning filesystem structure");
    }
    for moment in moment_output_spec {
        let folder = path_generator(moment);
        if !folder.exists() {
            if let Err(e) = fs::create_dir_all(&folder) {
                println!("Failed to create directory {}\nSkipping...", e);
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
    let path_generator = Arc::new(&path_generator);
    let export_job_generator = Arc::new(&export_job_generator);
    thread::scope(|s| {
        for chunk in moment_output_spec.chunks(chunk_size) {
            let total = Arc::clone(&total);
            let done = Arc::clone(&done);
            let path_generator = Arc::clone(&path_generator);
            let export_job_generator = Arc::clone(&export_job_generator);
            s.spawn(move || {
                for moment in chunk {
                    let output_folder = path_generator(moment);

                    for job in export_job_generator(moment) {
                        match job {
                            ExportJobSpec::ImageConvert {
                                output_file_name,
                                original_image_path,
                                output_format,
                            } => {
                                let (image_extension, lib_format) = match output_format {
                                    ImageFormat::Jpeg => ("jpeg".to_owned(), Some(image::ImageFormat::Jpeg)),
                                    ImageFormat::Png => ("png".to_owned(), Some(image::ImageFormat::Png)),
                                    ImageFormat::None => (original_image_path.extension()
                                          .map(|x| x.to_string_lossy().to_string())
                                          .unwrap_or("unknown".to_owned()) , 
                                          None)
                                };

                                if let Some(lib_format) = lib_format {
                                  let res_front = convert_to(
                                      &original_image_path,
                                      &output_folder.join(output_file_name + "." + &image_extension),
                                      lib_format,
                                  );
                                  print_if_err(&res_front);
                                } else {
                                    let res_bts = fs::copy(
                                      &original_image_path,
                                      output_folder.join(output_file_name + "." + &image_extension),
                                  );
                                  print_if_err(&res_bts);
                                }

                            }
                            ExportJobSpec::Copy {
                                output_file_name,
                                original_path,
                            } => {
                                let mb_ext = original_path.extension().and_then(|s| s.to_str());
                                if let Some(ext) = mb_ext {
                                    let res_bts = fs::copy(
                                        &original_path,
                                        output_folder.join(output_file_name + "." + ext),
                                    );
                                    print_if_err(&res_bts);
                                } else {
                                    println!(
                                        "Warning, no extension detected! {}",
                                        original_path.to_string_lossy()
                                    );
                                }
                            }
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
