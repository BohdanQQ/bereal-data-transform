use crate::{args::ImageFormat, BerealBTSData, OutputMomentSpec, OutputRealmojiSpec};
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

pub trait ExportJobGenerator {
    type ParamExportsT;
    type ParamFolderT;

    fn get_export_jobs(&self, inputs: &Self::ParamExportsT) -> Vec<ExportJobSpec>;
    fn get_output_folder(&self, inputs: &Self::ParamFolderT) -> PathBuf;
}

pub struct ExportParameters {
    pub input_path: PathBuf,
    pub image_format: ImageFormat,
}

impl<'a> ExportJobGenerator for OutputMomentSpec<'a> {
    type ParamExportsT = ExportParameters;
    type ParamFolderT = PathBuf;

    fn get_export_jobs(&self, params: &ExportParameters) -> Vec<crate::ExportJobSpec> {
        let mut result = vec![
            crate::ExportJobSpec::ImageConvert {
                output_file_name: self.file_name_prefix.clone() + "_camera_front",
                original_image_path: params.input_path.join(&self.moment.front_camera_path),
                output_format: params.image_format.clone(),
            },
            crate::ExportJobSpec::ImageConvert {
                output_file_name: self.file_name_prefix.clone() + "_camera_back",
                original_image_path: params.input_path.join(&self.moment.back_camera_path),
                output_format: params.image_format.clone(),
            },
        ];

        if let Some(BerealBTSData::Video { path }) = &self.moment.behind_the_scenes {
            result.push(crate::ExportJobSpec::Copy {
                output_file_name: self.file_name_prefix.clone() + "_BTS",
                original_path: params.input_path.join(path),
            });
        }
        result
    }

    fn get_output_folder(&self, output_folder_path: &PathBuf) -> PathBuf {
        output_folder_path.join(&self.folder)
    }
}

impl ExportJobGenerator for OutputRealmojiSpec {
    type ParamExportsT = ExportParameters;

    type ParamFolderT = PathBuf;

    fn get_export_jobs(&self, params: &Self::ParamExportsT) -> Vec<crate::ExportJobSpec> {
        vec![crate::ExportJobSpec::ImageConvert {
            output_file_name: self.file_name_prefix.clone(),
            original_image_path: params.input_path.join(&self.image_file),
            output_format: params.image_format.clone(),
        }]
    }

    fn get_output_folder(&self, output_folder: &Self::ParamFolderT) -> PathBuf {
        output_folder.join(self.folder.clone())
    }
}

pub fn export_generic<T, PathParam, ExportParam>(
    path_params: PathParam,
    export_params: ExportParam,
    output_specs: &Vec<T>,
    verbose: bool,
) -> usize
where
    T: Send + Sync + ExportJobGenerator<ParamExportsT = ExportParam, ParamFolderT = PathParam>,
    PathParam: Send + Sync,
    ExportParam: Send + Sync,
{
    let total = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);

    if verbose {
        println!("Spawning filesystem structure");
    }
    for item in output_specs {
        let folder = item.get_output_folder(&path_params);
        if !folder.exists() {
            if let Err(e) = fs::create_dir_all(&folder) {
                println!("Failed to create directory {}\nSkipping...", e);
                continue;
            }
        }
    }

    let cpu_count = max(1, num_cpus::get());
    let chunk_size = (output_specs.len() + cpu_count - 1) / cpu_count;
    let thread_count = output_specs.chunks(chunk_size).len();
    if verbose {
        println!(
            "Converting... 1-T Workload: {}, Thread Count: {}",
            chunk_size, thread_count
        );
    }

    let total = Arc::new(&total);
    let done = Arc::new(&done);
    let path_params = Arc::new(&path_params);
    let export_params = Arc::new(&export_params);
    thread::scope(|s| {
        for chunk in output_specs.chunks(chunk_size) {
            // acts as a "selective move"
            // only chunk is "really moved" rest is moved as references (Arc)
            let total = Arc::clone(&total);
            let done = Arc::clone(&done);
            let path_params = Arc::clone(&path_params);
            let export_params = Arc::clone(&export_params);
            s.spawn(move || {
                for item in chunk {
                    let output_folder = item.get_output_folder(path_params.as_ref());
                    let mut success = true;
                    for job in item.get_export_jobs(export_params.as_ref()) {
                        success &= match job {
                            ExportJobSpec::ImageConvert {
                                output_file_name,
                                original_image_path,
                                output_format,
                            } => export_image(
                                output_format,
                                original_image_path,
                                &output_folder,
                                output_file_name,
                            ),
                            ExportJobSpec::Copy {
                                output_file_name,
                                original_path,
                            } => perform_copy(original_path, &output_folder, output_file_name),
                        };
                    }
                    if success {
                        total.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
                done.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }
        let total = Arc::clone(&total);
        s.spawn(move || {
            while done.load(std::sync::atomic::Ordering::SeqCst) != thread_count {
                if verbose {
                    print!(
                        "\rProgress: {}/{} ({} threads done)",
                        total.load(std::sync::atomic::Ordering::SeqCst),
                        output_specs.len(),
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
                    output_specs.len(),
                    done.load(std::sync::atomic::Ordering::SeqCst)
                );
            }
        });
    });

    total.load(std::sync::atomic::Ordering::SeqCst)
}

fn perform_copy(
    original_path: PathBuf,
    output_folder: &Path,
    output_file_name_no_ext: String,
) -> bool {
    let mb_ext = original_path.extension().and_then(|s| s.to_str());
    if let Some(ext) = mb_ext {
        let target_path = &output_folder.join(output_file_name_no_ext + "." + ext);
        let input_path = &original_path;
        let res_bts = fs::copy(input_path, target_path);
        print_if_err(&res_bts, input_path, target_path)
    } else {
        println!(
            "Warning, no extension detected! {}",
            original_path.to_string_lossy()
        );
        false
    }
}

fn export_image(
    output_format: ImageFormat,
    original_image_path: PathBuf,
    output_folder: &Path,
    output_file_name_no_ext: String,
) -> bool {
    let (image_extension, lib_format) = match output_format {
        ImageFormat::Jpeg => ("jpeg".to_owned(), Some(image::ImageFormat::Jpeg)),
        ImageFormat::Jpg => ("jpg".to_owned(), Some(image::ImageFormat::Jpeg)),
        ImageFormat::Png => ("png".to_owned(), Some(image::ImageFormat::Png)),
        ImageFormat::None => (
            original_image_path
                .extension()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or("unknown".to_owned()),
            None,
        ),
    };

    let target_path = &output_folder.join(output_file_name_no_ext + "." + &image_extension);
    let input_path = &original_image_path;
    if let Some(lib_format) = lib_format {
        let res_front = convert_to(input_path, target_path, lib_format);
        print_if_err(&res_front, input_path, target_path)
    } else {
        let res_bts = fs::copy(input_path, target_path);
        print_if_err(&res_bts, input_path, target_path)
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

/// returns false on error
fn print_if_err<R, E: Display>(res: &Result<R, E>, from: &Path, to: &Path) -> bool {
    if let Err(e) = res {
        println!(
            "{} -> {} failed: {e}",
            from.to_string_lossy(),
            to.to_string_lossy()
        );
        return false;
    }
    true
}
