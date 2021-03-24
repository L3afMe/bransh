use std::{
    os::unix::process::ExitStatusExt,
    process::{Child, Command, Stdio},
};

use br_command::load_builtins;
use br_data::context::Context;
use br_parser::{tokenize_command, OutputType, TokenizationError};

#[allow(clippy::field_reassign_with_default)]
pub fn execute_once(command: String) {
    let mut ctx = Context::default();
    load_builtins(&mut ctx);
    ctx.cli.command_buffer = command;

    execute(&mut ctx);
}

pub fn execute(ctx: &mut Context) -> Option<i32> {
    let tokenize_res = tokenize_command(ctx.cli.command_buffer.clone(), ctx);
    let tokenized = match tokenize_res {
        Ok(val) => val,
        Err(why) => {
            match why {
                TokenizationError::UnterminateQuote(pos, chr) => {
                    eprintln!("Unterminated string ({}) at pos {}", chr, pos);
                },
                TokenizationError::InvalidEscape(pos, chr) => {
                    eprintln!("Invalid escape code (\\{}) at pos {}", chr, pos);
                },
                TokenizationError::UnexpectedValue(pos, val_exp, val_got) => {
                    eprintln!("Unexpected value at pos {}, expected {}, got {}", pos, val_exp, val_got);
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
    'cmdloop: while let Some(cmd) = commands.next() {
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
            _ => {
                for builtin in ctx.builtins.clone() {
                    if builtin.name == cmd.command {
                        output = (builtin.execute)(cmd.args.clone(), ctx);

                        last_output = output;
                        joiner = cmd.output_type;

                        continue 'cmdloop;
                    }
                }

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

                        match cmd_child.wait() {
                            Ok(exit_status) => match exit_status.code() {
                                Some(code) => {
                                    output = code;
                                },
                                None => match exit_status.signal() {
                                    Some(signal) => {
                                        output = 128 + signal;
                                    },
                                    None => {
                                        eprintln!("Status terminated with no exit status!");
                                        output = 0;
                                    },
                                },
                            },
                            Err(why) => {
                                eprintln!("Unable to execute command! {}", why);
                            },
                        }
                    },
                    Err(why) => {
                        eprintln!("Unable to execute command! {}", why);
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
