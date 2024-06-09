use clap::Parser;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum OutputFile {
    Json(PathBuf),
    Csv(PathBuf),
}

impl FromStr for OutputFile {
    type Err = String;

    fn from_str(input: &str) -> Result<OutputFile, Self::Err> {
        let path = Path::new(input);
        let extension = path.extension().and_then(|s| s.to_str());
        match extension {
            Some("json") => Ok(OutputFile::Json(path.into())),
            Some("csv") => Ok(OutputFile::Csv(path.into())),
            _ => Err(String::from(
                "The output file must have a .json or .csv extension",
            )),
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    version = "1.0",
    author = "Andrey <suzukenobi@gmail.com>",
    about = "Converts .xcactivitylog files to .json or .csv format"
)]
pub struct Args {
    #[clap(short, long, value_name = "FILE", value_parser =clap::value_parser!(PathBuf))]
    pub input: PathBuf,

    #[clap(short, long, value_name = "FILE", value_parser = clap::value_parser!(OutputFile))]
    pub output: OutputFile,
}
