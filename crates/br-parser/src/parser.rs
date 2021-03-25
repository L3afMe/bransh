use std::{fmt, ops::Range};

use br_data::context::Context;

use crate::{Command, lexer::Token};

use logos::Lexer;

pub type CommandList = Vec<Command>;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum ParseError {
    UnexpectedValue(Range<usize>, String, String),
    LexError(Range<usize>, String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::UnexpectedValue(range, expected_val, got_val) => 
                format!(
                    "Unexpected value at pos {}..{}, expected '{}' but got'{}'",
                    range.start, range.end, expected_val, got_val,
                ),
            Self::LexError(range, value) => 
                format!(
                    "Unable to parse input at pos {}..{}, unexpected value '{}'", 
                    range.start, range.end, value,
                )
        };
        write!(f, "{}", value)
    }
}

pub fn parse_lex(mut lex: Lexer<Token>, ctx: &Context) -> Result<CommandList, ParseError> {
    let mut cmd_list = CommandList::new();

    let mut arg_builder = String::new();
    let mut cmd_builder = Command::default();

    let mut last_token = None;
    while let Some(token) = lex.next() {
        match token.clone() {
            Token::Background => {
                let mut next_token = lex.clone().peekable();
                let mut is_last = false;
                if next_token.peek().is_some() {
                    if let Some(Token::Output(_)) = next_token.peek() {
                        is_last = true;
                    }
                } else {
                    is_last = true
                }

                if is_last {
                    cmd_builder.background = true;
                } else {
                    arg_builder.push('&');
                }
            },
            Token::Error => 
                return Err(ParseError::LexError(lex.span(), lex.slice().to_string())),
            Token::Variable((var_name, is_env)) => {
                let var_val = ctx.get_variable(&var_name, String::default(), is_env);
                arg_builder.push_str(&var_val);
            },
            Token::Word => {
                let word = lex.slice();
                let arg = if word.starts_with('~') {
                    let home = if word.starts_with("~/") || word == "~" {
                        let home_dir = match home::home_dir() {
                            Some(home_dir) => home_dir.to_string_lossy().to_string(),
                            None => String::from("~")
                        };

                        home_dir + word.strip_prefix("~").unwrap()
                    } else {
                        // TODO: Get other user home dir
                        word.to_string()
                    };

                    home
                } else {
                    word.to_string()
                };

                arg_builder.push_str(&arg);
            }
            Token::NumberLiteral => arg_builder.push_str(lex.slice()),
            Token::StringLiteral(val) => arg_builder.push_str(&val),
            Token::Output(out_type) => {
                if let Some(Token::Output(_)) = last_token {
                    return Err(ParseError::UnexpectedValue(
                        lex.span(),
                        String::from("command"),
                        lex.slice().to_string()
                    ));
                } else if last_token == None || cmd_builder.command.is_empty() {
                    return Err(ParseError::UnexpectedValue(
                        lex.span(),
                        String::from("command"),
                        lex.slice().to_string()
                    ));
                }

                cmd_builder.output_type = out_type;

                cmd_list.push(cmd_builder);
                cmd_builder = Command::default();
            },
            Token::Whitespace => { 
                if !arg_builder.is_empty() {
                    if cmd_builder.command.is_empty() {
                        cmd_builder.command = arg_builder;
                    } else {
                        cmd_builder.args.push(arg_builder);
                    }

                    arg_builder = String::new();
                }
            },
            Token::Comment => {}
        }

        last_token = Some(token);
    }

    if !arg_builder.is_empty() {
        if cmd_builder.command.is_empty() {
            cmd_builder.command = arg_builder;
        } else {
            cmd_builder.args.push(arg_builder);
        }
    }

    if !cmd_builder.command.is_empty() {
        cmd_list.push(cmd_builder);
    }

    Ok(cmd_list)
}

mod test {
    use logos::Logos;
    use br_data::context::Context;

    use super::{parse_lex, ParseError};
    use crate::lexer::Token;
    use crate::{Command, OutputType};

    fn get_output(command: &str) -> Result<Vec<Command>, ParseError> {
        let lex = Token::lexer(command);
        let ctx = Context::default();
        parse_lex(lex, &ctx)
    }

    #[test]
    fn one_echo() {
        assert_eq!(
            get_output("echo hi"),
            Ok(vec![
                Command {
                    command: String::from("echo"),
                    args: vec![
                        String::from("hi"),
                    ],
                    background: false,
                    output_type: OutputType::Ignore,
                },
            ])
        );
    }

    #[test]
    fn pipe_depend_not() {
        assert_eq!(
            get_output("echo This || echo \"Not this\""),
            Ok(vec![
                Command {
                    command: String::from("echo"),
                    args: vec![
                        String::from("This"),
                    ],
                    background: false,
                    output_type: OutputType::DependNot,
                },
                Command {
                    command: String::from("echo"),
                    args: vec![
                        String::from("Not this"),
                    ],
                    background: false,
                    output_type: OutputType::Ignore,
                },
            ])
        );
    }

    #[test]
    fn pipe_depend() {
        assert_eq!(
            get_output("echo This || echo \"And this\""),
            Ok(vec![
                Command {
                    command: String::from("echo"),
                    args: vec![
                        String::from("This"),
                    ],
                    background: false,
                    output_type: OutputType::Depend,
                },
                Command {
                    command: String::from("echo"),
                    args: vec![
                        String::from("And this"),
                    ],
                    background: false,
                    output_type: OutputType::Ignore,
                },
            ])
        );
    }

}
