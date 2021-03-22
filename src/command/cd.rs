use std::{env, path::Path};

pub fn execute(args: Vec<String>) -> i32 {
    let mut dirs = get_prev_dirs().unwrap_or_default();
    let mut dir_idx = get_dir_idx().unwrap_or(dirs.len() as usize);

    let mut new_dir = args.into_iter().peekable().peek().map_or("~", |dir| dir).to_string();
    match new_dir.as_ref() {
        "-" => {
            if dir_idx == 0 {
                println!("Already at end of dir history!");
                return 3;
            }

            dir_idx -= 1;
            match dirs.get(dir_idx){
                Some(dir) => new_dir = dir.clone(),
                None => {
                    println!("Unable to get previous dir!");
                    return 3;
                }
            }
        },
        "+" => {
            if dir_idx == dirs.len() {
                println!("Already at start of dir history!");
                return 3;
            }

            dir_idx += 1;
            match dirs.get(dir_idx){
                Some(dir) => new_dir = dir.clone(),
                None => {
                    println!("Unable to get next dir!");
                    return 3;
                }
            }
        },
        _ => {},
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

    if new_dir != "-" && new_dir != "+" {
        if dir_idx != dirs.len() {
            let _ = dirs.split_off(dir_idx);
        } else {
            dir_idx += 1;
        }
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

    2
}

fn get_dir_idx() -> Option<usize> {
    if let Ok(dir_idx_str) = env::var("BRSH_CUR_DIR_IDX") {
        if let Ok(dir_idx) = dir_idx_str.parse::<usize>() {
            return Some(dir_idx);
        }
    }

    None
}

fn get_prev_dirs() -> Option<Vec<String>> {
    if let Ok(dir) = env::var("BRSH_PREV_dirs") {
        return Some(dir.split(':').map(|dir| dir.to_string()).collect());
    }

    None
}

fn set_prev_dirs(dirs: Vec<String>) {
    env::set_var("BRSH_PREV_dirs", dirs.join(":"));
}

fn set_dir_idx(dir_idx: usize) {
    env::set_var("BRSH_CUR_DIR_IDX", dir_idx.to_string());
}
