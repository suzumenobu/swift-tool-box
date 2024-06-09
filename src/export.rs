use crate::{deser::XActivityLogObject, token::Token};
use std::io::Write;

pub fn to_csv(
    tokens: impl IntoIterator<Item = Token>,
    file: &mut impl Write,
) -> anyhow::Result<()> {
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

pub fn to_json(tokens: Vec<XActivityLogObject>, file: &mut impl Write) -> anyhow::Result<()> {
    let json_str = serde_json::to_string_pretty(&tokens)?;
    write!(file, "{}", json_str)?;
    Ok(())
}
