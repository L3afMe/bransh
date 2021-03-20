use std::{
    ffi::OsString,
    os::unix::{prelude::MetadataExt, process::ExitStatusExt},
    process::Command,
    str::SplitWhitespace,
};

use crate::{options::Options, print_line};

mod cd;

pub fn execute(_opts: &Options, cmd: String, args: SplitWhitespace) -> i32 {
    match cmd.as_ref() {
        "cd" => {
            return cd::execute(args);
        },
        _ => {
            let external_cmd = Command::new(cmd).args(args).spawn();
            match external_cmd {
                Ok(mut cmd) => match cmd.wait() {
                    Ok(exit_status) => match exit_status.code() {
                        Some(code) => return code,
                        None => match exit_status.signal() {
                            Some(signal) => return 128 + signal,
                            None => {
                                print_line(format!("Status terminated with no exit status!"));
                            },
                        },
                    },
                    Err(why) => {
                        print_line(format!("Unable to get exit status for command! {}", why));
                    },
                },
                Err(why) => {
                    print_line(format!("Unable to execute command! {}", why));
                },
            }
        },
    }

    0
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

        for file in files {
            if let Ok(file) = file {
                if file.file_name() != command {
                    continue;
                }
                if let Ok(metadata) = file.metadata() {
                    let mode = metadata.mode();
                    let can_exec = mode & 0o001 == 0o001;
                    if can_exec {
                        if file.file_name() == command {
                            return true;
                        }
                    }
                }
            }
        }
    }

    return false;
}

pub fn get_valid_commands(opts: &Options) -> Vec<OsString> {
    let mut cmds = Vec::new();

    for path in &opts.paths {
        let files = if let Ok(files) = path.read_dir() {
            files
        } else {
            continue;
        };

        for file in files {
            if let Ok(file) = file {
                if let Ok(metadata) = file.metadata() {
                    let mode = metadata.mode();
                    let can_exec = mode & 0o001 == 0o001;
                    if can_exec {
                        cmds.push(file.file_name())
                    }
                }
            }
        }
    }

    cmds
}
