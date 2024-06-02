use crate::token::Token;

pub trait XActivityLogClass<T>
where
    T: Iterator<Item = Token>,
    Self: Sized,
{
    fn from_tokens(
        tokens: &mut T,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self>;

    fn get_class_name() -> &'static str;
    fn count_parsed_tokens(&self) -> usize;
}

fn deser_exact<T, I>(tokens: &mut I, class_position_to_name: &mut Vec<String>) -> Option<T>
where
    T: XActivityLogClass<I>,
    I: Iterator<Item = Token>,
{
    loop {
        match tokens.next() {
            Some(Token::ClassName(name)) => class_position_to_name.push(name.to_string()),
            Some(Token::ClassInstance(position)) => {
                let got_class = &class_position_to_name[position - 1];
                let expected_class = T::get_class_name();
                log::debug!("Expected {expected_class} got {got_class}");
                // assert!(
                //     class_position_to_name[position - 1] == T::get_class_name(),
                //     "Expected {} got {}",
                //     T::get_class_name(),
                //     class_position_to_name[position - 1],
                // );
                return Some(T::from_tokens(tokens, class_position_to_name).unwrap());
            }
            None => {
                return None;
            }
            v => {
                log::warn!("Unexpected token: {v:?}");
                return None;
            }
        }
    }
}

fn deser_vec<T, I>(
    tokens: &mut I,
    capacity: usize,
    class_position_to_name: &mut Vec<String>,
) -> Vec<T>
where
    T: XActivityLogClass<I>,
    I: Iterator<Item = Token>,
{
    log::debug!("Parsing {} elements of {}", capacity, T::get_class_name());
    let mut result = Vec::with_capacity(capacity);
    for i in 0..capacity {
        log::debug!("Parsing {i} of {capacity}");
        match deser_exact::<T, _>(tokens, class_position_to_name) {
            Some(t) => result.push(t),
            None => break,
        }
    }
    result
}

#[derive(Debug)]
pub struct IDECommandLineBuildLog {
    pub section_type: i8,
    pub domain_type: String,
    pub title: String,
    pub signature: String,
    pub time_started_recording: f64,
    pub time_stopped_recording: f64,
    pub sub_sections: Vec<IDEActivityLogSection>,
}

impl<T> XActivityLogClass<T> for IDECommandLineBuildLog
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        let section_type = i8::from(tokens.next().unwrap());
        let domain_type = String::from(tokens.next().unwrap());
        let title = String::from(tokens.next().unwrap());
        let signature = String::from(tokens.next().unwrap());
        let time_started_recording = f64::from(tokens.next().unwrap());
        let time_stopped_recording = f64::from(tokens.next().unwrap());
        let sub_sections_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let sub_sections = deser_vec(tokens, sub_sections_size, class_position_to_name);

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

    fn get_class_name() -> &'static str {
        "IDEActivityLogSection"
    }

    fn count_parsed_tokens(&self) -> usize {
        6 + self
            .sub_sections
            .iter()
            .map(|s| <IDEActivityLogSection as XActivityLogClass<T>>::count_parsed_tokens(s))
            .sum::<usize>()
            + 1
            + 1
    }
}

#[derive(Debug)]
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
    pub unique_identifier: Option<String>,
    pub localized_result_string: Option<String>,
    pub xcbuild_signature: String,
    pub unknown1: Option<String>,
    pub unknown2: Option<String>,
    pub attachments: Vec<IDEActivityLogSectionAttachment>,
}

impl<T> XActivityLogClass<T> for IDEActivityLogSection
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        let section_type = i8::from(tokens.next().unwrap());
        let domain_type = String::from(tokens.next().unwrap());
        let title = String::from(tokens.next().unwrap());
        let signature = String::from(tokens.next().unwrap());
        let time_started_recording = f64::from(tokens.next().unwrap());
        let time_stopped_recording = f64::from(tokens.next().unwrap());
        let sub_sections_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let sub_sections = deser_vec(tokens, sub_sections_size, class_position_to_name);
        let text = Option::<String>::from(tokens.next().unwrap());
        let messages_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let messages = deser_vec(tokens, messages_size, class_position_to_name);
        let was_cancelled = bool::from(tokens.next().unwrap());
        let is_quiet = bool::from(tokens.next().unwrap());
        let was_fetched_from_cache = bool::from(tokens.next().unwrap());
        let subtitle = Option::<String>::from(tokens.next().unwrap());
        let unique_identifier = Option::<String>::from(tokens.next().unwrap());
        let localized_result_string = Option::<String>::from(tokens.next().unwrap());
        let xcbuild_signature = String::from(tokens.next().unwrap());
        let unknown1 = Option::<String>::from(tokens.next().unwrap());
        let unknown2 = Option::<String>::from(tokens.next().unwrap());
        let attachments_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let attachments = deser_vec(tokens, attachments_size, class_position_to_name);
        log::debug!("Attachments: {attachments:?}");

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
            unique_identifier,
            localized_result_string,
            xcbuild_signature,
            unknown1,
            unknown2,
            attachments,
            // unknown,
        })
    }

    fn get_class_name() -> &'static str {
        "IDEActivityLogSection"
    }

    fn count_parsed_tokens(&self) -> usize {
        6 + self
            .sub_sections
            .iter()
            .map(|s| <IDEActivityLogSection as XActivityLogClass<T>>::count_parsed_tokens(s))
            .sum::<usize>()
            + 1
            + 1
            + self
                .messages
                .iter()
                .map(|m| <IDEActivityLogMessage as XActivityLogClass<T>>::count_parsed_tokens(m))
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

#[derive(Default, Debug)]
pub struct IDEActivityLogMessage {
    pub title: String,
    pub short_title: Option<String>,
    pub time_emitted: u64,
    pub range_end_in_section_text: u64,
    pub range_start_in_section_text: u64,
    pub sub_messages: Vec<IDEActivityLogMessage>,
    pub severity: i32,
    pub r#type: String,
    pub location: Option<DVTDocumentLocation>,
    pub category_ident: String,
    pub secondary_locations: Vec<DVTDocumentLocation>,
    pub additional_description: Option<String>,
}

impl<T> XActivityLogClass<T> for IDEActivityLogMessage
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        let title = String::from(tokens.next().unwrap());
        let short_title = Option::<String>::from(tokens.next().unwrap());
        let time_emitted = u64::from(tokens.next().unwrap());
        let range_end_in_section_text = u64::from(tokens.next().unwrap());
        let range_start_in_section_text = u64::from(tokens.next().unwrap());
        let sub_messages_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let sub_messages = deser_vec(tokens, sub_messages_size, class_position_to_name);
        let severity = i32::from(tokens.next().unwrap());
        let r#type = String::from(tokens.next().unwrap());
        let location = deser_exact::<DVTDocumentLocation, _>(tokens, class_position_to_name);
        let category_ident = String::from(tokens.next().unwrap());
        let secondary_locations_size = Option::<usize>::from(tokens.next().unwrap()).unwrap_or(0);
        let secondary_locations =
            deser_vec(tokens, secondary_locations_size, class_position_to_name);
        let additional_description = Option::<String>::from(tokens.next().unwrap());
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

    fn get_class_name() -> &'static str {
        "IDEActivityLogMessage"
    }

    fn count_parsed_tokens(&self) -> usize {
        5 + self
            .sub_messages
            .iter()
            .map(|m| <IDEActivityLogMessage as XActivityLogClass<T>>::count_parsed_tokens(m))
            .sum::<usize>()
            + 1
            + 4
            + self
                .secondary_locations
                .iter()
                .map(|l| <DVTDocumentLocation as XActivityLogClass<T>>::count_parsed_tokens(l))
                .sum::<usize>()
            + 1
            + 1
    }
}

#[derive(Default, Debug)]
pub struct IDEActivityLogSectionAttachment {
    pub identifier: String,
    pub major_version: u64,
    pub minor_version: u64,
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
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        let identifier = String::from(tokens.next().unwrap());
        let major_version = u64::from(tokens.next().unwrap());
        let minor_version = u64::from(tokens.next().unwrap());
        Ok(Self {
            identifier,
            major_version,
            minor_version,
        })
    }

    fn get_class_name() -> &'static str {
        "IDEActivityLogSectionAttachment"
    }

    fn count_parsed_tokens(&self) -> usize {
        3
    }
}

#[derive(Default, Debug)]
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
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        for _ in 0..6 {
            tokens.next();
        }
        Ok(Self::default())
    }

    fn get_class_name() -> &'static str {
        "IDEActivityLogUnitTestSection"
    }

    fn count_parsed_tokens(&self) -> usize {
        6
    }
}

#[derive(Default, Debug)]
pub struct DVTDocumentLocation {
    pub document_url_string: String,
    pub timestamp: f64,
}

impl<T> XActivityLogClass<T> for DVTDocumentLocation
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        _class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        for _ in 0..2 {
            tokens.next();
        }
        Ok(Self::default())
    }

    fn get_class_name() -> &'static str {
        "DVTDocumentLocation"
    }

    fn count_parsed_tokens(&self) -> usize {
        2
    }
}

#[derive(Default, Debug)]
pub struct DVTTextDocumentLocation {
    pub starting_line_number: u64,
    pub starting_column_number: u64,
    pub ending_line_number: u64,
    pub ending_column_number: u64,
    pub character_range_end: u64,
    pub character_range_start: u64,
    pub location_encoding: u64,
}

impl<T> XActivityLogClass<T> for DVTTextDocumentLocation
where
    T: Iterator<Item = Token>,
{
    fn from_tokens(
        tokens: &mut T,
        _class_position_to_name: &mut Vec<String>,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Parsing {}",
            <Self as XActivityLogClass<T>>::get_class_name()
        );
        for _ in 0..7 {
            tokens.next();
        }
        Ok(Self::default())
    }

    fn get_class_name() -> &'static str {
        "DVTTextDocumentLocation"
    }

    fn count_parsed_tokens(&self) -> usize {
        7
    }
}

#[derive(Default, Debug)]
struct IDEActivityLogCommandInvocationSection {}

#[derive(Default, Debug)]
struct IDEActivityLogMajorGroupSection {}
