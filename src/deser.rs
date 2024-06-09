use std::iter::Peekable;

use crate::log_class::*;
use crate::token::Token;
use serde::Serialize;

#[derive(Serialize)]
pub enum XActivityLogObject {
    IDECommandLineBuildLog(IDECommandLineBuildLog),
    IDEActivityLogSection(IDEActivityLogSection),
    IDEActivityLogMessage(IDEActivityLogMessage),
    IDEActivityLogSectionAttachment(IDEActivityLogSectionAttachment),
    IDEActivityLogUnitTestSection(IDEActivityLogUnitTestSection),
    DVTDocumentLocation(DVTDocumentLocation),
    IDEActivityLogCommandInvocationSection(IDEActivityLogCommandInvocationSection),
}

pub struct Deserializer<'a, T>
where
    T: Iterator<Item = Token>,
{
    tokens: &'a mut Peekable<T>,
    class_position_to_name: Vec<String>,
}

impl<'a, T> Deserializer<'a, T>
where
    T: Iterator<Item = Token>,
{
    pub fn new(tokens: &'a mut Peekable<T>) -> Self {
        Self {
            tokens,
            class_position_to_name: vec![],
        }
    }
}

impl<'a, T> Iterator for Deserializer<'a, T>
where
    T: Iterator<Item = Token>,
{
    type Item = XActivityLogObject;

    fn next(&mut self) -> Option<Self::Item> {
        _deserialize(&mut self.tokens, &mut self.class_position_to_name)
    }
}

fn _deserialize<T>(
    tokens: &mut Peekable<T>,
    class_position_to_name: &mut Vec<String>,
) -> Option<XActivityLogObject>
where
    T: Iterator<Item = Token>,
{
    loop {
        let obj = match tokens.peek() {
            Some(Token::ClassInstance(position)) => {
                let class_name = &class_position_to_name[position - 1];
                log::debug!("Got instance of {class_name}");
                let obj = match class_name.as_str() {
                    "IDECommandLineBuildLog" => XActivityLogObject::IDECommandLineBuildLog(
                        IDECommandLineBuildLog::from_tokens(tokens, class_position_to_name)
                            .unwrap(),
                    ),
                    "IDEActivityLogSection" => XActivityLogObject::IDEActivityLogSection(
                        IDEActivityLogSection::from_tokens(tokens, class_position_to_name).unwrap(),
                    ),
                    "IDEActivityLogCommandInvocationSection" => {
                        XActivityLogObject::IDEActivityLogCommandInvocationSection(
                            IDEActivityLogCommandInvocationSection::from_tokens(
                                tokens,
                                class_position_to_name,
                            )
                            .unwrap(),
                        )
                    }
                    "IDEActivityLogMessage" | "IDEDiagnosticActivityLogMessage" => {
                        XActivityLogObject::IDEActivityLogMessage(
                            IDEActivityLogMessage::from_tokens(tokens, class_position_to_name)
                                .unwrap(),
                        )
                    }
                    "IDEActivityLogSectionAttachment" => {
                        XActivityLogObject::IDEActivityLogSectionAttachment(
                            IDEActivityLogSectionAttachment::from_tokens(
                                tokens,
                                class_position_to_name,
                            )
                            .unwrap(),
                        )
                    }
                    "IDEActivityLogUnitTestSection" => {
                        XActivityLogObject::IDEActivityLogUnitTestSection(
                            IDEActivityLogUnitTestSection::from_tokens(
                                tokens,
                                class_position_to_name,
                            )
                            .unwrap(),
                        )
                    }
                    "DVTDocumentLocation" => XActivityLogObject::DVTDocumentLocation(
                        DVTDocumentLocation::from_tokens(tokens, class_position_to_name).unwrap(),
                    ),
                    s => panic!("Unknwon class instance: {s:?}"),
                };
                Some(obj)
            }
            Some(Token::ClassName(_)) => {
                let name = tokens.next().unwrap().to_string();
                log::debug!("Got class name: {name}");
                class_position_to_name.push(name.to_string());
                continue;
            }
            None => {
                log::warn!("No more tokens to parse");
                None
            }
            v => {
                log::warn!("Unknwon value: {v:?}");
                tokens.next();
                continue;
            }
        };
        return obj;
    }
}
