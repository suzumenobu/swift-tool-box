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
    Array(Vec<Token>),
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

        let token = match token_type {
            // Example: `200#`
            // Left hand side value: An unsigned 64 bits integer.
            TokenType::Int => match lhs {
                Some(lhs) => Token::Int(lhs.parse::<u64>()?),
                None => bail!("Wrong format"),
            },
            // Example: `afd021ebae48c141^`
            // Left hand side value: A little-endian floating point number, encoded in hexadecimal.
            TokenType::Double => match lhs {
                Some(lhs) => {
                    let bytes: [u8; 8] = u64::from_str_radix(&lhs, 16)?.to_le_bytes();
                    let data: f64 = f64::from_le_bytes(bytes);
                    Token::Double(data)
                }
                None => bail!("Wrong format"),
            },
            // Example: `21%IDEActivityLogSection`
            // Left hand side value: An `Integer` with the number of characters that are part of the `Class name`.
            // Right hand side value: The characters that are part of the `Class name`
            TokenType::ClassName => match lhs {
                Some(lhs) => Token::ClassName(lhs),
                None => bail!("Wrong format"),
            },
            // TODO: The following comment is wrong, there could be a string too
            // Example: `2@`
            // Left hand side value: An `Integer` with the index of the `Class name` of the `Class instance`'s type.
            TokenType::ClassInstance => match lhs {
                Some(lhs) => Token::ClassInstance(lhs),
                None => bail!("Wrong format"),
            },
            // Example: `5"Hello`
            // Left hand side value: An `Integer` with the number of characters that are part of the `String`.
            // Right hand side value: The characters that are part of the `String`
            TokenType::String => match lhs {
                Some(lhs) => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    log::debug!("Read string: {:?}", data);
                    Token::String(data)
                }
                None => bail!("Wrong format"),
            },
            // No left, nor right hand side value
            TokenType::Null => Token::Null,
            // Example: `22(`
            // Left hand side value: An `Integer` with the number of elements that are part of the `Array`.
            TokenType::Array => match lhs {
                Some(lhs) => {
                    let size = lhs.parse::<usize>()?;
                    let elements = (0..size)
                        .filter_map(|_| self.scan_token().ok())
                        .collect::<Vec<_>>();
                    Token::Array(elements)
                }
                None => bail!("Wrong format"),
            },
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
