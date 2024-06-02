use crate::log_class::*;
use crate::token::Token;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum XActivityLogObject {
    IDECommandLineBuildLog(IDECommandLineBuildLog),
    IDEActivityLogSection(IDEActivityLogSection),
    IDEActivityLogMessage(IDEActivityLogMessage),
    IDEActivityLogSectionAttachment(IDEActivityLogSectionAttachment),
    IDEActivityLogUnitTestSection(IDEActivityLogUnitTestSection),
    DVTDocumentLocation(DVTDocumentLocation),
    DVTTextDocumentLocation(DVTTextDocumentLocation),
}

pub fn deserialize<T>(tokens: &mut T) -> Vec<XActivityLogObject>
where
    T: Iterator<Item = Token>,
{
    let mut class_position_to_name: Vec<String> = Vec::new();
    let mut result = Vec::new();
    loop {
        let token = tokens.next();
        let obj = match token {
            Some(Token::ClassInstance(position)) => {
                let class_name = &class_position_to_name[position - 1];
                log::debug!("Got instance of {class_name}");
                match class_name.as_str() {
                    "IDECommandLineBuildLog" => XActivityLogObject::IDECommandLineBuildLog(
                        IDECommandLineBuildLog::from_tokens(tokens, &mut class_position_to_name)
                            .unwrap(),
                    ),
                    "IDEActivityLogSection" => XActivityLogObject::IDEActivityLogSection(
                        IDEActivityLogSection::from_tokens(tokens, &mut class_position_to_name)
                            .unwrap(),
                    ),
                    "IDEActivityLogMessage" => XActivityLogObject::IDEActivityLogMessage(
                        IDEActivityLogMessage::from_tokens(tokens, &mut class_position_to_name)
                            .unwrap(),
                    ),
                    "IDEActivityLogSectionAttachment" => {
                        XActivityLogObject::IDEActivityLogSectionAttachment(
                            IDEActivityLogSectionAttachment::from_tokens(
                                tokens,
                                &mut class_position_to_name,
                            )
                            .unwrap(),
                        )
                    }
                    "IDEActivityLogUnitTestSection" => {
                        XActivityLogObject::IDEActivityLogUnitTestSection(
                            IDEActivityLogUnitTestSection::from_tokens(
                                tokens,
                                &mut class_position_to_name,
                            )
                            .unwrap(),
                        )
                    }
                    "DVTDocumentLocation" => XActivityLogObject::DVTDocumentLocation(
                        DVTDocumentLocation::from_tokens(tokens, &mut class_position_to_name)
                            .unwrap(),
                    ),
                    "DVTTextDocumentLocation" => XActivityLogObject::DVTTextDocumentLocation(
                        DVTTextDocumentLocation::from_tokens(tokens, &mut class_position_to_name)
                            .unwrap(),
                    ),
                    s => panic!("Unknwon class instance: {s:?}"),
                }
            }
            Some(Token::ClassName(name)) => {
                log::debug!("Got class name: {name}");
                class_position_to_name.push(name.to_string());
                continue;
            }
            None => break,
            v => {
                log::warn!("Unknwon value: {v:?}");
                continue;
            }
        };
        result.push(obj);
    }

    result
}
