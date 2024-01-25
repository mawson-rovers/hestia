use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use flate2::Compression;
use flate2::write::GzEncoder;
use glob::glob;

use crate::payload::Config;

pub fn zip_logs(config: &Config) {
    let path = config.log_path.as_ref().expect("UTS_LOG_PATH should be set");
    let path = format!("{}/*.csv", path);
    for file in glob(&path).expect("Glob pattern failed").flatten() {
        zip_file(file);
    }
}

fn zip_file(in_file: PathBuf) {
    let out_file = in_file.with_extension("csv.gz");
    if out_file.exists() && mtime(&out_file) >= mtime(&in_file) {
        // ignore files we've already compressed
        return
    }

    if let Ok(in_file) = File::open(in_file) {
        let mut input = BufReader::new(in_file);
        let output = File::create(out_file).expect("Should be able to write output file");
        let mut output = BufWriter::new(GzEncoder::new(output, Compression::fast()));
        let _ = io::copy(&mut input, &mut output);
    }
}

fn mtime(file: &Path) -> u64 {
    assert!(file.exists(), "file should exist: {}", file.display());
    // why is this so hard?! ¯\_(ツ)_/¯
    file.metadata().unwrap()
        .modified().unwrap()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap()
        .as_secs()
}