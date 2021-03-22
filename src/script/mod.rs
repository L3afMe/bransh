use std::{fs::OpenOptions, io::{Read, Write}, path::{Path, PathBuf}};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::{cli::history::get_config_dir, command::execute, prelude::Context};
use crate::cli::util::print_line;

pub fn load_rc(ctx: &mut Context) {
    let config_dir = match get_config_dir() {
        Some(dir) => Path::new(&dir).join("branshrc.br"),
        None => {
            eprintln!("Unable to get config directory!");
            return;
        }
    };

    if !config_dir.exists() {
        write_default_config(config_dir);
        return;
    }
    
    let mut config_file = match OpenOptions::new().read(true).open(config_dir) {
        Ok(config) => config,
        Err(why) => {
            eprintln!("Unable to read branshrc.br! {}", why);
            return;
        }
    };

    let mut config = String::new();
    match config_file.read_to_string(&mut config) {
        Ok(_) => {},
        Err(why) => {
            eprintln!("Unable to read branshrc.br! {}", why);
            return;
        }
    };

    if let Err(why) = enable_raw_mode() {
        println!("Unable to enable raw mode! {}", why);
    }

    for (line_num, line) in config.lines().enumerate() {
        ctx.command_buffer = line.to_string();
        if let Some(exit_code) = execute(ctx) {
            if exit_code != 0 {
                print_line(ctx, "Non 0 exit code returned while running file!");
                print_line(ctx, format!("Line {}: '{}'", line_num, line));
                return;
            }
        } else {
            // 'exit' executed
            return;
        }
    }

    if let Err(why) = disable_raw_mode() {
        print_line(ctx, format!("Unable to disable raw mode! {}", why));
    }
}

fn write_default_config(file: PathBuf) {
    let mut config = match OpenOptions::new().write(true).create(true).open(file) {
        Ok(conf) => conf,
        Err(why) => {
            eprintln!("Unable to create default branshrc.br! {}", why);
            return;
        }
    };

    let default_config = 
    r#"
set PROMPT "{WD} | "

set P_HOME_TRUNC true
set P_HOME_CHAR  "~"

set P_DIR_TRUNC 2
set P_DIR_CHAR  "…"

set SYN_HIGHLIGHTING true
"#.trim();

    match config.write_all(default_config.as_bytes()) {
        Ok(()) => {},
        Err(why) => eprintln!("Unable to write default config to branshrc.br! {}", why),
    };
}
