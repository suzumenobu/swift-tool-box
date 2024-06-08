use std::iter::Peekable;

use crate::token::Token;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::fmt;

// Define a custom error type
#[derive(Debug)]
struct NoMoreTokensError;

// Implement the Display trait for the custom error type
impl fmt::Display for NoMoreTokensError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No more tokens")
    }
}

// Implement the Error trait for the custom error type
impl Error for NoMoreTokensError {}

macro_rules! read_token {
    ($iter:expr) => {
        match $iter.next() {
            Some(val) => Ok(val),
            None => Err(NoMoreTokensError),
        }
    };
}

pub trait XActivityLogClass<T>
where
    T: Iterator<Item = Token>,
    Self: Sized,
{
    fn from_tokens(
        tokens: &mut T,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self>;

    fn get_possible_class_names() -> &'static [&'static str];
    fn count_parsed_tokens(&self) -> usize;
}

fn deser_exact<T, I>(
    tokens: &mut Peekable<I>,
    class_position_to_name: &mut Vec<String>,
) -> Option<T>
where
    T: XActivityLogClass<Peekable<I>>,
    I: Iterator<Item = Token>,
{
    loop {
        match tokens.peek() {
            Some(Token::ClassName(_)) => {
                class_position_to_name.push(read_token!(tokens).unwrap().to_string())
            }
            Some(Token::ClassInstance(position)) => {
                let got_class = &class_position_to_name[position - 1];
                let expected_classes = T::get_possible_class_names();
                log::debug!("Expected {expected_classes:?} got {got_class}");
                return Some(T::from_tokens(tokens, class_position_to_name).unwrap());
            }
            Some(Token::Null) | Some(Token::Json(_)) => {
                tokens.next();
                return None;
            }
            v => {
                panic!("Unexpected token: {v:?}");
            }
        }
    }
}

fn deser_vec<T, I>(
    tokens: &mut Peekable<I>,
    capacity: usize,
    class_position_to_name: &mut Vec<String>,
) -> Vec<T>
where
    T: XActivityLogClass<Peekable<I>>,
    I: Iterator<Item = Token>,
{
    let mut result = Vec::with_capacity(capacity);
    for i in 0..capacity {
        log::debug!(
            "[{:?}] Start parsing {} of {capacity}",
            T::get_possible_class_names(),
            i + 1,
        );
        match deser_exact::<T, _>(tokens, class_position_to_name) {
            Some(t) => result.push(t),
            None => break,
        }
    }
    result
}

#[derive(Debug, Serialize)]
pub struct IDECommandLineBuildLog {
    pub section_type: i8,
    pub domain_type: String,
    pub title: String,
    pub signature: String,
    pub time_started_recording: f64,
    pub time_stopped_recording: f64,
    pub sub_sections: Vec<IDEActivityLogSection>,
}

impl<T> XActivityLogClass<Peekable<T>> for IDECommandLineBuildLog
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut Peekable<T>,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Start parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        let section_type = i8::try_from(read_token!(tokens)?)?;
        let domain_type = String::try_from(read_token!(tokens)?)?;
        let title = String::try_from(read_token!(tokens)?)?;
        let signature = String::try_from(read_token!(tokens)?)?;
        let time_started_recording = f64::try_from(read_token!(tokens)?)?;
        let time_stopped_recording = f64::try_from(read_token!(tokens)?)?;
        let sub_sections_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let sub_sections = deser_vec(tokens, sub_sections_size, class_position_to_name);
        log::info!(
            "End of parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );

        Ok(Self {
            section_type,
            domain_type,
            title,
            signature,
            time_started_recording,
            time_stopped_recording,
            sub_sections,
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["IDEActivityLogSection"]
    }

    fn count_parsed_tokens(&self) -> usize {
        6 + self
            .sub_sections
            .iter()
            .map(|s| {
                <IDEActivityLogSection as XActivityLogClass<Peekable<T>>>::count_parsed_tokens(s)
            })
            .sum::<usize>()
            + 1
            + 1
    }
}

#[derive(Debug, Serialize)]
pub struct IDEActivityLogSection {
    pub section_type: i8,
    pub domain_type: String,
    pub title: String,
    pub signature: String,
    pub time_started_recording: f64,
    pub time_stopped_recording: f64,
    pub sub_sections: Vec<IDEActivityLogSection>,
    pub text: Option<String>,
    pub messages: Vec<IDEActivityLogMessage>,
    pub was_cancelled: bool,
    pub is_quiet: bool,
    pub was_fetched_from_cache: bool,
    pub subtitle: Option<String>,
    pub location: Option<DVTDocumentLocation>,
    pub command_details_spect: Option<String>,
    pub unique_identifier: Option<String>,
    pub localized_result_string: Option<String>,
    pub xcbuild_signature: Option<String>,
    pub attachments: Vec<IDEActivityLogSectionAttachment>,
    pub unknown: Option<u64>,
}

impl<T> XActivityLogClass<Peekable<T>> for IDEActivityLogSection
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut Peekable<T>,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        let section_type = i8::try_from(read_token!(tokens)?)?;
        let domain_type = String::try_from(read_token!(tokens)?)?;
        let title = String::try_from(read_token!(tokens)?)?;
        let signature = String::try_from(read_token!(tokens)?)?;
        let time_started_recording = f64::try_from(read_token!(tokens)?)?;
        let time_stopped_recording = f64::try_from(read_token!(tokens)?)?;
        let sub_sections_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let sub_sections = deser_vec(tokens, sub_sections_size, class_position_to_name);
        let text = Option::<String>::try_from(read_token!(tokens)?)?;
        let messages_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let messages = deser_vec(tokens, messages_size, class_position_to_name);
        let was_cancelled = bool::try_from(read_token!(tokens)?)?;
        let is_quiet = bool::try_from(read_token!(tokens)?)?;
        let was_fetched_from_cache = bool::try_from(read_token!(tokens)?)?;
        let subtitle = Option::<String>::try_from(read_token!(tokens)?)?;
        let location = deser_exact::<DVTDocumentLocation, _>(tokens, class_position_to_name);
        let command_details_spect = Option::<String>::try_from(read_token!(tokens)?)?;
        let unique_identifier = Option::<String>::try_from(read_token!(tokens)?)?;
        let localized_result_string = Option::<String>::try_from(read_token!(tokens)?)?;
        let xcbuild_signature = Option::<String>::try_from(read_token!(tokens)?)?;
        let mut attachments_found = false;
        let attachments_size = match tokens.peek() {
            Some(Token::Array(_)) => {
                attachments_found = true;
                Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0)
            }
            _ => 0,
        };
        let attachments = deser_vec(tokens, attachments_size, class_position_to_name);
        let mut unknown_found = false;
        let unknown = match tokens.peek() {
            Some(Token::Int(_)) | Some(Token::Null) if attachments_found => {
                unknown_found = true;
                Option::<u64>::try_from(read_token!(tokens)?)?
            }
            _ => None,
        };

        Ok(Self {
            section_type,
            domain_type,
            title,
            signature,
            time_started_recording,
            time_stopped_recording,
            sub_sections,
            text,
            messages,
            was_cancelled,
            is_quiet,
            was_fetched_from_cache,
            subtitle,
            location,
            command_details_spect,
            unique_identifier,
            localized_result_string,
            xcbuild_signature,
            attachments,
            unknown,
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &[
            "IDEActivityLogSection",
            "IDEActivityLogCommandInvocationSection",
        ]
    }

    fn count_parsed_tokens(&self) -> usize {
        6 + self
            .sub_sections
            .iter()
            .map(|s| {
                <IDEActivityLogSection as XActivityLogClass<Peekable<T>>>::count_parsed_tokens(s)
            })
            .sum::<usize>()
            + 1
            + 1
            + self
                .messages
                .iter()
                .map(|m| {
                    <IDEActivityLogMessage as XActivityLogClass<Peekable<T>>>::count_parsed_tokens(
                        m,
                    )
                })
                .sum::<usize>()
            + 1
            + 9
            + self
                .attachments
                .iter()
                .map(|a| {
                    <IDEActivityLogSectionAttachment as XActivityLogClass<T>>::count_parsed_tokens(
                        a,
                    )
                })
                .sum::<usize>()
            + 1
    }
}

#[derive(Default, Debug, Serialize)]
pub struct IDEActivityLogMessage {
    pub title: String,
    pub short_title: Option<String>,
    pub time_emitted: u64,
    pub range_end_in_section_text: u64,
    pub range_start_in_section_text: u64,
    pub sub_messages: Vec<IDEActivityLogMessage>,
    pub severity: i32,
    pub r#type: Option<String>,
    pub location: Option<DVTDocumentLocation>,
    pub category_ident: Option<String>,
    pub secondary_locations: Vec<DVTDocumentLocation>,
    pub additional_description: Option<String>,
}

impl<T> XActivityLogClass<Peekable<T>> for IDEActivityLogMessage
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut Peekable<T>,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        let title = String::try_from(read_token!(tokens)?)?;
        let short_title = Option::<String>::try_from(read_token!(tokens)?)?;
        let time_emitted = u64::try_from(read_token!(tokens)?)?;
        let range_end_in_section_text = u64::try_from(read_token!(tokens)?)?;
        let range_start_in_section_text = u64::try_from(read_token!(tokens)?)?;
        let sub_messages_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let sub_messages = deser_vec(tokens, sub_messages_size, class_position_to_name);
        let severity = i32::try_from(read_token!(tokens)?)?;
        let r#type = Option::<String>::try_from(read_token!(tokens)?)?;
        let location = deser_exact::<DVTDocumentLocation, _>(tokens, class_position_to_name);
        let category_ident = Option::<String>::try_from(read_token!(tokens)?)?;
        let secondary_locations_size =
            Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let secondary_locations =
            deser_vec(tokens, secondary_locations_size, class_position_to_name);
        let additional_description = Option::<String>::try_from(read_token!(tokens)?)?;

        Ok(Self {
            title,
            short_title,
            time_emitted,
            range_end_in_section_text,
            range_start_in_section_text,
            sub_messages,
            severity,
            r#type,
            location,
            category_ident,
            secondary_locations,
            additional_description,
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["IDEActivityLogMessage", "IDEDiagnosticActivityLogMessage"]
    }

    fn count_parsed_tokens(&self) -> usize {
        5 + self
            .sub_messages
            .iter()
            .map(|m| {
                <IDEActivityLogMessage as XActivityLogClass<Peekable<T>>>::count_parsed_tokens(m)
            })
            .sum::<usize>()
            + 1
            + 4
            + self
                .secondary_locations
                .iter()
                .map(|l| {
                    <DVTDocumentLocation as XActivityLogClass<Peekable<T>>>::count_parsed_tokens(l)
                })
                .sum::<usize>()
            + 1
            + 1
    }
}

#[derive(Default, Debug, Serialize)]
pub struct IDEActivityLogSectionAttachment {
    pub identifier: String,
    pub major_version: u64,
    pub minor_version: u64,
    pub unknown1: Value,
}

impl<T> XActivityLogClass<T> for IDEActivityLogSectionAttachment
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        _class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<T>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        let identifier = String::try_from(read_token!(tokens)?)?;
        let major_version = u64::try_from(read_token!(tokens)?)?;
        let minor_version = u64::try_from(read_token!(tokens)?)?;
        let unknown1 = Value::try_from(read_token!(tokens)?)?;
        log::info!(
            "End of parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        Ok(Self {
            identifier,
            major_version,
            minor_version,
            unknown1,
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["IDEActivityLogSectionAttachment"]
    }

    fn count_parsed_tokens(&self) -> usize {
        3
    }
}

#[derive(Default, Debug, Serialize)]
pub struct IDEActivityLogUnitTestSection {
    pub tests_passed_string: String,
    pub duration_string: String,
    pub summary_string: String,
    pub suite_name: String,
    pub test_name: String,
    pub performance_test_output_string: String,
}

impl<T> XActivityLogClass<T> for IDEActivityLogUnitTestSection
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        _class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<T>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        for _ in 0..6 {
            tokens.next();
        }
        Ok(Self::default())
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["IDEActivityLogUnitTestSection"]
    }

    fn count_parsed_tokens(&self) -> usize {
        6
    }
}

#[derive(Debug, Serialize)]
pub enum DVTDocumentLocation {
    Base(DVTBaseDocumentLocation),
    Text(DVTTextDocumentLocation),
    Member(DVTMemberDocumentLocation),
}

impl<T> XActivityLogClass<Peekable<T>> for DVTDocumentLocation
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut Peekable<T>,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        let class_instance = usize::try_from(read_token!(tokens)?)?;
        let class_name = &class_position_to_name[class_instance - 1];

        let document_url_string = String::try_from(read_token!(tokens)?)?;
        let timestamp = f64::try_from(read_token!(tokens)?)?;

        let base = DVTBaseDocumentLocation {
            document_url_string,
            timestamp,
        };

        Ok(match class_name.as_str() {
            "DVTDocumentLocation" => DVTDocumentLocation::Base(base),
            "DVTTextDocumentLocation" => {
                for _ in 0..7 {
                    tokens.next();
                }
                DVTDocumentLocation::Text(DVTTextDocumentLocation::default())
            }
            "DVTMemberDocumentLocation" => {
                tokens.next();
                DVTDocumentLocation::Member(DVTMemberDocumentLocation::default())
            }
            _ => panic!("Unknwon class name"),
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["DVTDocumentLocation"]
    }

    fn count_parsed_tokens(&self) -> usize {
        2
    }
}

#[derive(Default, Debug, Serialize)]
pub struct DVTBaseDocumentLocation {
    pub document_url_string: String,
    pub timestamp: f64,
}

#[derive(Default, Debug, Serialize)]
pub struct DVTTextDocumentLocation {
    pub base: DVTBaseDocumentLocation,
    pub starting_line_number: u64,
    pub starting_column_number: u64,
    pub ending_line_number: u64,
    pub ending_column_number: u64,
    pub character_range_end: u64,
    pub character_range_start: u64,
    pub location_encoding: u64,
}

#[derive(Default, Debug, Serialize)]
pub struct DVTMemberDocumentLocation {
    pub base: DVTBaseDocumentLocation,
    pub member: String,
}

#[derive(Debug, Serialize)]
pub struct IDEActivityLogCommandInvocationSection {
    pub section_type: i8,
    pub domain_type: String,
    pub title: String,
    pub signature: String,
    pub time_started_recording: f64,
    pub time_stopped_recording: f64,
    pub sub_sections: Vec<IDEActivityLogSection>,
    pub text: Option<String>,
    pub messages: Vec<IDEActivityLogMessage>,
    pub was_cancelled: bool,
}

impl<T> XActivityLogClass<Peekable<T>> for IDEActivityLogCommandInvocationSection
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut Peekable<T>,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {:?}",
            <Self as XActivityLogClass<Peekable<T>>>::get_possible_class_names()
        );
        let _class_instance = usize::try_from(read_token!(tokens)?)?;
        let section_type = i8::try_from(read_token!(tokens)?)?;
        let domain_type = String::try_from(read_token!(tokens)?)?;
        let title = String::try_from(read_token!(tokens)?)?;
        let signature = String::try_from(read_token!(tokens)?)?;
        let time_started_recording = f64::try_from(read_token!(tokens)?)?;
        let time_stopped_recording = f64::try_from(read_token!(tokens)?)?;
        let sub_sections_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let sub_sections = deser_vec(tokens, sub_sections_size, class_position_to_name);
        let text = Option::<String>::try_from(read_token!(tokens)?)?;
        let messages_size = Option::<usize>::try_from(read_token!(tokens)?)?.unwrap_or(0);
        let messages = deser_vec(tokens, messages_size, class_position_to_name);
        let was_cancelled = bool::try_from(read_token!(tokens)?)?;

        Ok(Self {
            section_type,
            domain_type,
            title,
            signature,
            time_started_recording,
            time_stopped_recording,
            sub_sections,
            text,
            messages,
            was_cancelled,
        })
    }

    fn get_possible_class_names() -> &'static [&'static str] {
        &["IDEActivityLogCommandInvocationSection"]
    }

    fn count_parsed_tokens(&self) -> usize {
        todo!()
    }
}
