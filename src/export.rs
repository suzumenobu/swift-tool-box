use crate::{deser::XActivityLogObject, token::Token};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub fn to_csv(tokens: impl IntoIterator<Item = Token>, path: &PathBuf) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut file = BufWriter::new(file);
    writeln!(file, "type,value")?;
    let mut counter = 0;
    for token in tokens.into_iter() {
        writeln!(file, "{},{}", token.get_type_as_str(), token.to_string())?;
        counter += 1;

        if counter % 1000 == 0 {
            log::debug!("Writter {counter} lines");
        }
    }
    Ok(())
}

pub fn to_json(
    tokens: impl IntoIterator<Item = XActivityLogObject>,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut file = BufWriter::new(file);

    writeln!(file, "{}", "[")?;

    let mut logs = tokens.into_iter().peekable();
    loop {
        match logs.peek() {
            Some(_) => {
                let json_str = serde_json::to_string_pretty(&logs.next().unwrap())?;
                if logs.peek().is_some() {
                    writeln!(file, "{},", json_str)?;
                } else {
                    writeln!(file, "{}", json_str)?;
                }
            }
            None => break,
        }
    }

    writeln!(file, "{}", "]")?;
    Ok(())
}
