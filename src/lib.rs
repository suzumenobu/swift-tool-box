use flate2::read::GzDecoder;
use std::{fs::File, io, path::PathBuf};

pub mod deser;
pub mod export;
mod log_class;
pub mod parser;
mod token;

/// Reads a gzipped file
pub fn read_gzipped_file(path: &PathBuf) -> io::Result<GzDecoder<File>> {
    let file = File::open(path)?;
    Ok(GzDecoder::new(file))
}
