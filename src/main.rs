use anyhow::bail;
use flate2::bufread::GzDecoder;
use log;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

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
            _ => bail!("Unknown char: {value}"),
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

struct Scanner<T>
where
    T: Read,
{
    contents: BufReader<T>,
}

impl<T> Scanner<T>
where
    T: Read,
{
    pub fn new(contents: T) -> Self {
        Self {
            contents: BufReader::new(contents),
        }
    }

    pub fn scan_token(&mut self) -> anyhow::Result<Token> {
        let (lhs, token_type) = self.scan_lhs_and_token_type()?;

        log::debug!("Left hand side: {lhs:?}");
        let token = match token_type {
            TokenType::Int => match lhs {
                Some(lhs) => Token::Int(lhs.parse::<u64>()?),
                None => bail!("Wrong format"),
            },
            TokenType::Double => match lhs {
                Some(lhs) => {
                    let bytes: [u8; 8] = u64::from_str_radix(&lhs, 16).unwrap().to_le_bytes();
                    let data: f64 = f64::from_le_bytes(bytes);
                    Token::Double(data)
                }
                None => bail!("Wrong format"),
            },
            TokenType::ClassName => match lhs {
                Some(lhs) => Token::ClassName(lhs),
                None => bail!("Wrong format"),
            },
            TokenType::ClassInstance => match lhs {
                Some(lhs) => Token::ClassInstance(lhs),
                None => bail!("Wrong format"),
            },
            TokenType::String => match lhs {
                Some(lhs) => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    log::debug!("Readed string {data:?}");
                    Token::String(data)
                }
                None => bail!("Wrong format"),
            },
            TokenType::Null => Token::Null,
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

fn read_gzipped_file(path: &str) -> io::Result<GzDecoder<impl BufRead>> {
    let file = BufReader::new(File::open(path)?);
    Ok(GzDecoder::new(file))
}

fn main() {
    env_logger::init();
    let path = "./static/test.xcactivitylog";
    let contents = read_gzipped_file(path).unwrap();
    let mut scanner = Scanner::new(contents);
    scanner.scan_header().unwrap();
    let mut counter = 0;
    loop {
        let token = scanner.scan_token().unwrap();
        log::info!("{token:?}");
        counter += 1;
        println!("{counter} tokens reaad");
    }
}
