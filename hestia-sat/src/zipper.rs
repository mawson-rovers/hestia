use std::fs::{File, create_dir_all};
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use flate2::Compression;
use flate2::write::GzEncoder;
use glob::glob;
use log::{error, info};

use crate::payload::Config;

pub fn zip_logs(config: &Config) {
    let path = config.log_path.as_ref().expect("UTS_LOG_PATH should be set");
    let download_path: PathBuf = config.download_path.as_ref().expect("UTS_DOWNLOAD_PATH should be set").into();
    if !download_path.exists() {
        info!("Creating download directory at {}", download_path.display());
        create_dir_all(&download_path).expect(&*format!("Could not create download directory {}", download_path.display()));
    }
    let path = format!("{}/*.csv", path);
    for file in glob(&path).expect("Glob pattern failed").flatten() {
        zip_file(file, &download_path);
    }
}

fn zip_file(in_path: PathBuf, download_path: &PathBuf) {
    let mut out_path = download_path.clone();
    out_path.push(in_path.file_name().unwrap());
    out_path.set_extension("csv.gz");

    if out_path.exists() {
        let out_mtime = match mtime(&out_path) {
            Ok(mtime) => mtime,
            Err(err) => {
                error!("Could not read mtime of {}: {}", out_path.display(), err);
                return;
            }
        };
        let in_mtime = match mtime(&in_path) {
            Ok(mtime) => mtime,
            Err(err) => {
                error!("Could not read mtime of {}: {}", in_path.display(), err);
                return;
            }
        };
        if out_mtime >= in_mtime {
            // ignore files we've already compressed
            info!("Not compressing {} as it has already been compressed", in_path.display());
            return
        }
    }

    if let Ok(in_file) = File::open(&in_path) {
        let mut input = BufReader::new(in_file);
        let output = match File::create(&out_path) {
            Ok(file) => file,
            Err(err) => {
                error!("Could not create file {}: {}", out_path.display(), err);
                return;
            }
        };
        let mut output = BufWriter::new(GzEncoder::new(output, Compression::fast()));
        if let Err(err) = io::copy(&mut input, &mut output) {
            error!("Could not write compressed stream for {}: {}", in_path.display(), err);
            return
        }
    }
}

fn mtime(file: &Path) ->  io::Result<u64> {
    assert!(file.exists(), "file should exist: {}", file.display());
    let modified =
        file.metadata()?
        .modified()?;
    match modified.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => Ok(duration.as_secs()),
        Err(err) => Err(io::Error::other(err.to_string()))
    }
}