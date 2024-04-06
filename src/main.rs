/// SLF paser
/// Documentation for SLF could be found here:
/// https://github.com/MobileNativeFoundation/XCLogParser/blob/master/docs/Xcactivitylog%20Format.md
use anyhow::bail;
use flate2::bufread::GzDecoder;
use log;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug)]
enum TokenType {
    Int,
    Double,
    ClassName,
    ClassInstance,
    String,
    Null,
    Array,
}

impl TryFrom<char> for TokenType {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use TokenType::*;
        Ok(match value {
            '#' => Int,
            '^' => Double,
            '-' => Null,
            '"' => String,
            '(' => Array,
            '%' => ClassName,
            '@' => ClassInstance,
            _ => bail!("Unknown char: {}", value),
        })
    }
}

#[derive(Debug)]
enum Token {
    Int(u64),
    Double(f64),
    ClassName(String),
    ClassInstance(String),
    String(String),
    Null,
    Array(usize),
}

/// Main struct for SLF parsing
struct Parser<T>
where
    T: Read,
{
    contents: BufReader<T>,
}

impl<T> Parser<T>
where
    T: Read,
{
    pub fn new(contents: T) -> Self {
        Self {
            contents: BufReader::new(contents),
        }
    }

    /// Scans token from the `contents`
    /// Fails on wrong SLF format or EOF
    pub fn scan_token(&mut self) -> anyhow::Result<Token> {
        let (lhs, token_type) = self.scan_lhs_and_token_type()?;
        log::debug!("Left hand side: {:?}, Token type: {:?}", lhs, token_type);

        let token = match (token_type, lhs) {
            // No left, nor right hand side value
            (TokenType::Null, None) => Token::Null,
            (token_type, Some(lhs)) => match token_type {
                // Example: `200#`
                // Left hand side value: An unsigned 64 bits integer.
                TokenType::Int => Token::Int(lhs.parse::<u64>()?),

                // Example: `afd021ebae48c141^`
                // Left hand side value: A little-endian floating point number, encoded in hexadecimal.
                TokenType::Double => {
                    let bytes: [u8; 8] = u64::from_str_radix(&lhs, 16)?.to_le_bytes();
                    let data: f64 = f64::from_le_bytes(bytes);
                    Token::Double(data)
                }

                // Example: `21%IDEActivityLogSection`
                // Left hand side value: An `Integer` with the number of characters that are part of the `Class name`.
                // Right hand side value: The characters that are part of the `Class name`
                TokenType::ClassName => Token::ClassName(lhs),

                // TODO: The following comment is wrong, there could be a string too
                // Example: `2@`
                // Left hand side value: An `Integer` with the index of the `Class name` of the `Class instance`'s type.
                TokenType::ClassInstance => Token::ClassInstance(lhs),

                // Example: `5"Hello`
                // Left hand side value: An `Integer` with the number of characters that are part of the `String`.
                // Right hand side value: The characters that are part of the `String`
                TokenType::String => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    log::debug!("Read string: {:?}", data);
                    Token::String(data)
                }

                // Example: `22(`
                // Left hand side value: An `Integer` with the number of elements that are part of the `Array`.
                TokenType::Array => Token::Array(lhs.parse::<usize>()?),

                TokenType::Null => bail!("Wrong SLF format. Got Null and some lhs"),
            },
            _ => bail!("Wrong token type and lhs combo."),
        };

        Ok(token)
    }

    /// Scans the left hand side value and determine the token type
    fn scan_lhs_and_token_type(&mut self) -> anyhow::Result<(Option<String>, TokenType)> {
        let mut lhs = String::new();
        let mut buf = [0; 1];

        loop {
            self.contents.read_exact(&mut buf)?;
            let value = buf[0] as char;
            if let Ok(token_type) = TokenType::try_from(value) {
                log::debug!("Got {} token type", value);
                let payload = if lhs.is_empty() { None } else { Some(lhs) };
                return Ok((payload, token_type));
            }

            lhs.push(value);
        }
    }

    /// Reads `SLF0` header
    fn scan_header(&mut self) -> io::Result<()> {
        let mut buf = [0; 4];
        self.contents.read_exact(&mut buf)?;
        Ok(())
    }
}

/// Reads a gzipped file
fn read_gzipped_file(path: &str) -> io::Result<GzDecoder<impl BufRead>> {
    let file = BufReader::new(File::open(path)?);
    Ok(GzDecoder::new(file))
}

fn main() {
    env_logger::init();

    let path = "./static/test.xcactivitylog";

    let contents = read_gzipped_file(path).unwrap();
    let mut parser = Parser::new(contents);

    parser.scan_header().unwrap();

    let mut counter = 0;
    loop {
        let token = parser.scan_token().unwrap();
        log::info!("{:?}", token);
        counter += 1;
        println!("{} tokens read", counter);
    }
}
