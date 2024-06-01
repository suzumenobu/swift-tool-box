/// SLF paser
/// Documentation for SLF could be found here:
/// https://github.com/MobileNativeFoundation/XCLogParser/blob/master/docs/Xcactivitylog%20Format.md
use anyhow::bail;
use flate2::read::GzDecoder;
use log;
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read};

mod export;

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

impl ToString for Token {
    fn to_string(&self) -> String {
        use Token::*;
        match self {
            Int(v) => v.to_string(),
            Double(v) => v.to_string(),
            ClassName(v) => v.to_string(),
            ClassInstance(v) => v.to_string(),
            String(v) => v.to_string(),
            Null => "null".to_string(),
            Array(v) => v.to_string(),
        }
    }
}

impl Token {
    fn get_type_as_str(&self) -> &str {
        use Token::*;
        match self {
            Int(_) => "int",
            Double(_) => "double",
            ClassName(_) => "class_name",
            ClassInstance(_) => "class_instance",
            String(_) => "string",
            Null => "null",
            Array(_) => "array",
        }
    }
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
                TokenType::ClassName => {
                    let size = lhs.parse::<usize>()?;
                    let mut buf = vec![0; size];
                    self.contents.read_exact(&mut buf)?;
                    let data = String::from_utf8(buf)?;
                    log::debug!("Read string: {:?}", data);
                    Token::ClassName(data)
                }

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

    fn iter(&mut self) -> ParserIterator<T> {
        self.scan_header().unwrap();
        ParserIterator { parser: self }
    }
}

struct ParserIterator<'a, T>
where
    T: Read,
{
    parser: &'a mut Parser<T>,
}

impl<'a, T> Iterator for ParserIterator<'a, T>
where
    T: Read,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.scan_token() {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}

trait XActivitylogItem<T>
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(&self, tokens: T, class_position_to_name: &mut Vec<String>);
}

struct IDEActivityLogSection {
    section_type: i8,
    domain_type: String,
    title: String,
    signature: String,
    time_started_recording: f64,
    time_stopped_recording: f64,
    sub_sections: Vec<IDEActivityLogSection>,
    text: String,
    messages: Vec<IDEActivityLogMessage>,
    was_cancelled: bool,
    is_quiet: bool,
    was_fetched_from_cache: bool,
    subtitle: String,
    unique_identifier: String,
    localized_result_string: String,
    xcbuild_signature: String,
    attachments: Vec<IDEActivityLogSectionAttachment>,
    unknonw: i32,
}

struct IDEActivityLogMessage {
    title: String,
    short_title: String,
    time_emitted: f32,
    range_end_in_section_text: u64,
    range_start_in_section_text: u64,
    sub_message: Vec<IDEActivityLogMessage>,
    severity: i32,
    r#type: String,
    location: DVTDocumentLocation,
    category_ident: String,
    secondary_location: Vec<DVTDocumentLocation>,
    additional_description: String,
}

struct IDEActivityLogSectionAttachment {
    utime: u64,
    stilsle: u64,
    max_rss: u64,
    wc_start_time: u64,
    wc_duration: u64,
}

struct IDEActivityLogUnitTestSection {
    tests_passed_string: String,
    duration_string: String,
    summary_string: String,
    suite_name: String,
    test_name: String,
    performance_test_output_string: String,
}

struct DVTDocumentLocation {
    document_url_string: String,
    timestamp: f64,
}

struct DVTTextDocumentLocation {
    starting_line_number: u64,
    starting_column_number: u64,
    ending_line_number: u64,
    ending_column_number: u64,
    character_range_end: u64,
    character_range_start: u64,
    location_encoding: u64,
}

struct IDEActivityLogCommandInvocationSection {}

struct IDEActivityLogMajorGroupSection {}

fn serialize<T>(tokens: T) -> Vec<Box<dyn XActivitylogItem<T>>>
where
    T: Iterator<Item = Token>,
{
    todo!()
}

/// Reads a gzipped file
fn read_gzipped_file(path: &str) -> io::Result<GzDecoder<File>> {
    let file = File::open(path)?;
    Ok(GzDecoder::new(file))
}

fn main() {
    env_logger::init();

    let path = "./static/1.xcactivitylog";

    let contents = read_gzipped_file(path).unwrap();
    let mut parser = Parser::new(contents);

    let mut file = File::create("result.csv").unwrap();
    export::to_csv(parser.iter(), &mut file).unwrap();
}
