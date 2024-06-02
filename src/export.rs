use crate::Token;
use std::io::Write;

pub fn to_csv(
    tokens: impl IntoIterator<Item = Token>,
    file: &mut impl Write,
) -> anyhow::Result<()> {
    write!(file, "type,value\n")?;
    let mut counter = 0;
    for token in tokens.into_iter() {
        write!(file, "{},{}\n", token.get_type_as_str(), token.to_string())?;
        counter += 1;

        if counter % 1000 == 0 {
            log::debug!("Writter {counter} lines");
        }
    }
    Ok(())
}
