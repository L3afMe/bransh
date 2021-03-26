use std::{os::unix::process::ExitStatusExt, path::PathBuf, process::{Child, Command, Stdio}, str::FromStr};

use br_command::load_builtins;
use br_data::context::Context;
use br_parser::{OutputType, parse_command};

#[allow(clippy::field_reassign_with_default)]
pub fn execute_once(command: String) {
    let mut ctx = Context::default();
    load_builtins(&mut ctx);
    ctx.cli.command_buffer = command;

    execute(&mut ctx);
}

pub fn execute(ctx: &mut Context) -> Option<i32> {
    let commands_wrapped = parse_command(ctx.cli.command_buffer.clone(), ctx);
    let commands = match commands_wrapped {
        Ok(cmds) => cmds,
        Err(why) => {
            eprintln!("{}", why);

            return Some(-1);
        },
    };

    if commands.is_empty() {
        return Some(0);
    }

    let mut last_command = None;
    let mut last_output = 0;
    let mut joiner = OutputType::Ignore;

    let mut commands = commands.into_iter().peekable();
    'cmdloop: while let Some(mut cmd) = commands.next() {
        match joiner {
            OutputType::Ignore
                | OutputType::Pipe => {},
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
            OutputType::Redirect => {},
            OutputType::RedirectAppend => {}
        }
        let mut output = 0;

        match cmd.command.as_ref() {
            "exit" => return None,
            _ => {
                if cmd.command.starts_with('.') || cmd.command.starts_with('/') {
                    let file = PathBuf::from_str(&cmd.command).unwrap();
                    if file.exists() && file.is_dir() {
                        cmd.command = String::from("cd");
                        cmd.args.insert(0, file.to_str().unwrap().to_string());
                    }
                }

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
