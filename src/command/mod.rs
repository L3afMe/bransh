use std::{
    os::unix::{prelude::MetadataExt, process::ExitStatusExt},
    process::Command,
    time::Duration,
};

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};

use crate::{cli::util::print_line, options::Options, prelude::Context};

mod cd;

pub fn execute(ctx: &mut Context) -> Option<i32> {
    let trimmed_command = ctx.command_buffer.trim();
    if trimmed_command.is_empty() {
        return Some(0);
    }

    let mut cmd_split = trimmed_command.split_whitespace();
    let cmd = cmd_split.next().unwrap();
    let args = cmd_split;

    let mut output = 0;

    match cmd {
        "exit" => return None,
        "cd" => {
            output = cd::execute(args);
        },
        _ => {
            let external_cmd = Command::new(cmd).args(args).spawn();
            match external_cmd {
                Ok(mut cmd) => loop {
                    let poll_res = poll(Duration::from_millis(10));
                    if let Ok(has_event) = poll_res {
                        if has_event {
                            let event = match read() {
                                Ok(event) => event,
                                Err(why) => {
                                    print_line(format!("Unable to capture event! {}", why));
                                    continue;
                                },
                            };

                            if let Event::Key(key) = event {
                                if let KeyCode::Char(chr) = key.code {
                                    if chr == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                        if let Err(why) = cmd.kill() {
                                            print_line(format!("Unable to terminate command! {}", why));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    match cmd.try_wait() {
                        Ok(exit_status) => {
                            if let Some(exit_status) = exit_status {
                                match exit_status.code() {
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
                                            print_line("Status terminated with no exit status!");
                                        },
                                    },
                                }
                            }
                        },
                        Err(why) => {
                            print_line(format!("Unable to get commant status command! {}", why));
                        },
                    }
                },
                Err(why) => {
                    print_line(format!("Unable to execute command! {}", why));
                },
            }
        },
    }

    Some(output)
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
