use std::{env, path::Path};

use crate::prelude::Context;

pub fn execute(args: Vec<String>, _ctx: &mut Context) -> i32 {
    let mut dirs = get_prev_dirs().unwrap_or_default();
    let mut dir_idx = get_dir_idx().unwrap_or(dirs.len() as usize);

    let original_dir = args.into_iter().peekable().peek().map_or("~", |dir| dir).to_string();
    let mut new_dir;
    match original_dir.as_ref() {
        "-" => {
            if dir_idx == 0 {
                println!("Already at end of dir history!");
                return 0;
            }

            dir_idx -= 1;
            match dirs.get(dir_idx) {
                Some(dir) => new_dir = dir.clone(),
                None => {
                    println!("Unable to get previous dir!");
                    return 0;
                },
            }
        },
        "+" => {
            if dir_idx == dirs.len() {
                println!("Already at start of dir history!");
                return 0;
            }

            dir_idx += 1;
            match dirs.get(dir_idx) {
                Some(dir) => new_dir = dir.clone(),
                None => {
                    println!("Unable to get next dir!");
                    return 0;
                },
            }
        },
        _ => {
            new_dir = original_dir.clone();
        },
    }

    if new_dir.starts_with('~') {
        match home::home_dir() {
            Some(home_dir) => {
                new_dir.remove(0);
                if let Some(home) = home_dir.to_str() {
                    new_dir = format!("{}{}", home, &new_dir);
                } else {
                    println!("Unable to get home directory!");
                    return 1;
                }
            },
            None => {
                println!("Unable to get home directory!");
                return 1;
            },
        }
    }

    let old_dir = env::current_dir();
    let path = Path::new(&new_dir);

    if let Err(why) = env::set_current_dir(path) {
        println!("Unable to move to directory! {}", why);
        return 3;
    }

    if original_dir != "-" && original_dir != "+" {
        if dir_idx != dirs.len() {
            let _ = dirs.split_off(dir_idx);
        }

        dir_idx += 1;
    }

    match old_dir {
        Ok(old) => dirs.push(old.to_str().unwrap().to_string()),
        Err(why) => {
            println!("Unable to save old dir! {}", why);
            return 2;
        },
    }

    set_prev_dirs(dirs);
    set_dir_idx(dir_idx);

    0
}

fn get_dir_idx() -> Option<usize> {
    let dir_idx = env::var("BRANSH_CUR_DIR_IDX").ok()?;
    dir_idx.parse::<usize>().ok()
}

fn get_prev_dirs() -> Option<Vec<String>> {
    let dir = env::var("BRANSH_PREV_DIRS").ok()?;
    Some(dir.split(':').map(|dir| dir.to_string()).collect())
}

fn set_prev_dirs(dirs: Vec<String>) {
    env::set_var("BRANSH_PREV_DIRS", dirs.join(":"));
}

fn set_dir_idx(dir_idx: usize) {
    env::set_var("BRANSH_CUR_DIR_IDX", dir_idx.to_string());
}
