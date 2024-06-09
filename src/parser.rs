use anyhow::bail;
use std::io::{self, BufReader, Read};

use crate::token::{Token, TokenType};

/// Main struct for SLF parsing
pub struct Parser<T>
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
        log::trace!("Left hand side: {:?}, Token type: {:?}", lhs, token_type);

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
                    let bytes = (0..lhs.len())
                        .step_by(2)
                        .filter_map(|i| u8::from_str_radix(&lhs[i..i + 2], 16).ok())
                        .collect::<Vec<_>>();
                    let arr: [u8; 8] = bytes.try_into().unwrap();
                    let data = f64::from_le_bytes(arr);
                    Token::Double(data)
                }

                // Example: `21%IDEActivityLogSection`
                // Left hand side value: An `Integer` with the number of characters that are part of the `Class name`.
                // Right hand side value: The characters that are part of the `Class name`
                TokenType::ClassName => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    Token::ClassName(data)
                }

                // TODO: The following comment is wrong, there could be a string too
                // Example: `2@`
                // Left hand side value: An `Integer` with the index of the `Class name` of the `Class instance`'s type.
                TokenType::ClassInstance => Token::ClassInstance(lhs.parse::<usize>()?),

                // Example: `5"Hello`
                // Left hand side value: An `Integer` with the number of characters that are part of the `String`.
                // Right hand side value: The characters that are part of the `String`
                TokenType::String => {
                    log::debug!("Lhs: {lhs}");
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    log::trace!("Read string: {:?}", data);
                    Token::String(data)
                }

                // Example: `22(`
                // Left hand side value: An `Integer` with the number of elements that are part of the `Array`.
                TokenType::Array => Token::Array(lhs.parse::<usize>()?),
                TokenType::Json => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    Token::Json(data)
                }

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
                log::trace!("Got {} token type", value);
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

    pub fn iter(&mut self) -> ParserIterator<T> {
        self.scan_header().unwrap();
        ParserIterator {
            parser: self,
            token_idx: 0,
        }
    }
}

pub struct ParserIterator<'a, T>
where
    T: Read,
{
    parser: &'a mut Parser<T>,
    token_idx: usize,
}

impl<'a, T> Iterator for ParserIterator<'a, T>
where
    T: Read,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.parser.scan_token();
        log::debug!("[{}]: {token:?}", self.token_idx);
        self.token_idx += 1;
        match token {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}
