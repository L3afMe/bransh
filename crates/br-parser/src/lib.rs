use std::{env, fs::Metadata, os::unix::prelude::MetadataExt, path::PathBuf, str::FromStr};

use br_data::context::Context;
use lexer::Token;
use logos::Logos;
use parser::{CommandList, ParseError, parse_lex};

pub mod lexer;
pub mod parser;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum OutputType {
    Ignore,
    Pipe,
    Depend,
    DependNot,
    Redirect,
    RedirectAppend,
}

impl Default for OutputType {
    fn default() -> Self {
        Self::Ignore
    }
}

#[derive(Debug, Default, PartialEq, Clone, Eq, Hash)]
pub struct Command {
    pub command:     String,
    pub args:        Vec<String>,
    pub background:  bool,
    pub output_type: OutputType,
}

pub fn parse_command(command: String, ctx: &Context) -> Result<CommandList, ParseError> {
    let lex = Token::lexer(&command);
    parse_lex(lex, ctx)
}

#[cfg(unix)]
pub fn can_exec(md: Metadata) -> bool {
    let mode = md.mode();
    mode & 0o001 == 0o001
}

#[cfg(windows)]
pub fn can_exec(md: Metadata) -> bool {
    // TODO: Windows perms
    true
}

pub fn is_valid_command(command: &str, ctx: &Context) -> bool {
    if command.starts_with('.') || command.starts_with('/') {
        let file = PathBuf::from_str(command).unwrap();
        if file.exists() {
            if let Ok(metadata) = file.metadata() {
                if (file.is_file() && can_exec(metadata)) || file.is_dir() {
                    return true;
                }
            }
        }
    }

    if ctx.builtins.clone().into_iter().any(|b| b.name == command) || ctx.aliases.contains_key(command) {
        return true;
    }

    let paths = match env::var("PATH") {
        Ok(paths) => paths,
        Err(_) => return false,
    };

    let paths = env::split_paths(&paths);

    for path in paths {
        let files = if let Ok(files) = path.read_dir() {
            files
        } else {
            continue;
        };

        for file in files.flatten() {
            if file.file_name() != command {
                continue;
            }
            if let Ok(metadata) = file.metadata() {
                if can_exec(metadata) && file.file_name() == command {
                    return true;
                }
            }
        }
    }

    false
}

pub fn get_valid_commands(ctx: &Context) -> Vec<String> {
    let mut cmds = Vec::new();

    let mut builtins: Vec<String> = ctx.builtins.clone().into_iter().map(|b| b.name.to_string()).collect();
    cmds.append(&mut builtins);

    let mut aliases: Vec<String> = ctx.aliases.keys().map(|alias| alias.to_string()).collect();
    cmds.append(&mut aliases);

    let paths = match env::var("PATH") {
        Ok(paths) => paths,
        Err(_) => return cmds,
    };

    let paths = env::split_paths(&paths);

    for path in paths {
        let files = if let Ok(files) = path.read_dir() {
            files
        } else {
            continue;
        };

        for file in files.flatten() {
            if let Ok(metadata) = file.metadata() {
                let mode = metadata.mode();
                let can_exec = mode & 0o001 == 0o001;
                if can_exec {
                    if let Some(file_name) = file.file_name().to_str() {
                        cmds.push(file_name.to_string())
                    }
                }
            }
        }
    }

    cmds.sort();

    cmds
}
