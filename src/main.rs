/// SLF paser
/// Documentation for SLF could be found here:
/// https://github.com/MobileNativeFoundation/XCLogParser/blob/master/docs/Xcactivitylog%20Format.md
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

mod cli;
mod deser;
mod export;
mod log_class;
mod parser;
mod token;

/// Reads a gzipped file
fn read_gzipped_file(path: &PathBuf) -> io::Result<GzDecoder<File>> {
    let file = File::open(path)?;
    Ok(GzDecoder::new(file))
}

fn main() {
    env_logger::init();

    let args = <cli::Args as clap::Parser>::parse();

    let contents = read_gzipped_file(&args.input).unwrap();
    let mut parser = parser::Parser::new(contents);

    match args.output {
        cli::OutputFile::Json(path) => {
            let mut tokens = parser.iter().peekable();
            let result = deser::Deserializer::new(&mut tokens);
            export::to_json(result, &path).unwrap();
        }
        cli::OutputFile::Csv(path) => {
            export::to_csv(parser.iter(), &path).unwrap();
        }
    }
}
