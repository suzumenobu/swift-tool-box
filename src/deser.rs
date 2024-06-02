use crate::log_class::*;
use crate::token::Token;

pub fn deserialize<T>(tokens: &mut T)
where
    T: Iterator<Item = Token>,
{
    // let mut result = Vec::new();
    let mut class_position_to_name: Vec<String> = Vec::new();
    loop {
        let token = tokens.next();
        match token {
            Some(Token::ClassInstance(position)) => {
                let class_name = &class_position_to_name[position - 1];
                log::debug!("Got instance of {class_name}");
                match class_name.as_str() {
                    "IDECommandLineBuildLog" => {
                        let obj = IDECommandLineBuildLog::from_tokens(
                            tokens,
                            &mut class_position_to_name,
                        );
                        println!("{obj:?}");
                    }
                    "IDEActivityLogSection" => {
                        let obj =
                            IDEActivityLogSection::from_tokens(tokens, &mut class_position_to_name);
                        println!("{obj:?}");
                    }
                    "IDEActivityLogMessage" => {
                        let obj =
                            IDEActivityLogMessage::from_tokens(tokens, &mut class_position_to_name);
                        println!("{obj:?}");
                    }
                    "IDEActivityLogSectionAttachment" => {
                        let obj = IDEActivityLogSectionAttachment::from_tokens(
                            tokens,
                            &mut class_position_to_name,
                        );
                        println!("{obj:?}");
                    }
                    "IDEActivityLogUnitTestSection" => {
                        let obj = IDEActivityLogUnitTestSection::from_tokens(
                            tokens,
                            &mut class_position_to_name,
                        );
                        println!("{obj:?}");
                    }
                    "DVTDocumentLocation" => {
                        let obj =
                            DVTDocumentLocation::from_tokens(tokens, &mut class_position_to_name);
                        println!("{obj:?}");
                    }
                    "DVTTextDocumentLocation" => {
                        let obj = DVTTextDocumentLocation::from_tokens(
                            tokens,
                            &mut class_position_to_name,
                        );
                        println!("{obj:?}");
                    }
                    s => panic!("Unknwon class instance: {s:?}"),
                };
            }
            Some(Token::ClassName(name)) => {
                log::debug!("Got class name: {name}");
                class_position_to_name.push(name.to_string());
            }
            None => break,
            v => {
                log::warn!("Unknwon value: {v:?}");
                continue;
            }
        }
    }
}
