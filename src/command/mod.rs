use std::{
    os::unix::{prelude::MetadataExt, process::ExitStatusExt},
    process::{Child, Command, Stdio},
};

use self::tokenize::{tokenize_command, OutputType, TokenizationError};
use crate::{cli::util::print_line, options::Options, prelude::Context};

mod cd;
pub mod tokenize;

pub fn execute(ctx: &mut Context) -> Option<i32> {
    let tokenize_res = tokenize_command(ctx.command_buffer.clone());
    let tokenized = match tokenize_res {
        Ok(val) => val,
        Err(why) => {
            match why {
                TokenizationError::UnterminateQuote(pos, chr) => {
                    print_line(ctx, format!("Unterminated string ({}) at pos {}", chr, pos));
                },
                TokenizationError::InvalidEscape(pos, chr) => {
                    print_line(ctx, format!("Invalid escape code (\\{}) at pos {}", chr, pos));
                },
                TokenizationError::UnexpectedValue(pos, val_exp, val_got) => {
                    print_line(
                        ctx,
                        format!("Unexpected value at pos {}, expected {}, got {}", pos, val_exp, val_got),
                    );
                },
            }
            return Some(-1);
        },
    };

    if tokenized.is_empty() {
        return Some(0);
    }

    let mut last_command = None;
    let mut last_output = 0;
    let mut joiner = OutputType::Ignore;

    let mut commands = tokenized.into_iter().peekable();
    while let Some(cmd) = commands.next() {
        match joiner {
            OutputType::Ignore => {},
            OutputType::Depend => {
                if last_output != 0 {
                    continue;
                }
            },
            OutputType::DependNot => {
                if last_output == 0 {
                    continue;
                }
            },
            OutputType::Pipe => {},
        }
        let mut output = 0;

        match cmd.command.as_ref() {
            "exit" => return None,
            "cd" => output = cd::execute(cmd.args),
            _ => {
                let mut external_cmd_builder = Command::new(cmd.command);
                external_cmd_builder.args(cmd.args);

                let stdin = last_command.map_or(Stdio::inherit(), |output: Child| Stdio::from(output.stdout.unwrap()));
                external_cmd_builder.stdin(stdin);

                if cmd.background {
                    external_cmd_builder.stdout(Stdio::null());
                } else {
                    let stdout = if commands.peek().is_some() && cmd.output_type == OutputType::Pipe {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    external_cmd_builder.stdout(stdout);
                }

                let external_cmd = external_cmd_builder.spawn();
                match external_cmd {
                    Ok(mut cmd_child) => {
                        if cmd.output_type == OutputType::Pipe {
                            last_command = Some(cmd_child);
                            continue;
                        }

                        last_command = None;
                        if cmd.background {
                            continue;
                        }

                        loop {
                            match cmd_child.wait() {
                                Ok(exit_status) => match exit_status.code() {
                                    Some(code) => {
                                        output = code;
                                        break;
                                    },
                                    None => match exit_status.signal() {
                                        Some(signal) => {
                                            output = 128 + signal;
                                            break;
                                        },
                                        None => {
                                            print_line(ctx, "Status terminated with no exit status!");
                                        },
                                    },
                                },
                                Err(why) => {
                                    print_line(ctx, format!("Unable to execute command! {}", why));
                                },
                            }
                        }
                    },
                    Err(why) => {
                        print_line(ctx, format!("Unable to execute command! {}", why));
                        last_command = None;
                    },
                }
            },
        }

        last_output = output;
        joiner = cmd.output_type;
    }

    Some(last_output)
}

pub fn is_valid_command(opts: &Options, command: &str) -> bool {
    if command == "exit" || command == "cd" {
        return true;
    }
    for path in &opts.paths {
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
                let mode = metadata.mode();
                let can_exec = mode & 0o001 == 0o001;
                if can_exec && file.file_name() == command {
                    return true;
                }
            }
        }
    }

    false
}

pub fn get_valid_commands(opts: &Options) -> Vec<String> {
    let mut cmds = vec![String::from("exit"), String::from("cd")];

    for path in &opts.paths {
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

    cmds
}
