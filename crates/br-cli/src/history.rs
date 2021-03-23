use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Error, Read, Write},
    path::Path,
};

use br_data::{context::{CommandBufferBackup, Context}, get_config_dir};
use crossterm::event::KeyCode;

use crate::util::{print_cmd_buf, print_error, restore_backup};

fn get_history_file(opts: &mut OpenOptions) -> Result<File, Error> {
    let conf_dir = get_config_dir();
    if conf_dir.is_none() {
        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            "Unable to find config directory",
        ));
    }

    let conf = conf_dir.unwrap();
    let path = Path::new(&conf);
    if !path.exists() {
        create_dir_all(path)?;
    }

    let history_path = path.join(".history");
    // print_error(ctx, format!("Path: {}",
    // history_path.to_str().unwrap()));
    opts.open(history_path)
}

pub fn init_history() -> Result<(), Error> {
    get_history_file(OpenOptions::new().write(true).create(true))?;

    Ok(())
}

pub fn find_history(start_match: String) -> Result<Vec<String>, Error> {
    let mut file = get_history_file(OpenOptions::new().read(true).write(true).create(true))?;

    let mut lines = String::new();
    file.read_to_string(&mut lines)?;

    let mut out_lines = Vec::new();
    for line in lines.split('\n') {
        if line.starts_with(&start_match) && !line.is_empty() {
            out_lines.push(line.to_string());
        }
    }

    // Remove consecutive duplicates
    out_lines.dedup();

    Ok(out_lines)
}

pub fn add_history(line: String) -> Result<(), Error> {
    let mut file = get_history_file(OpenOptions::new().append(true).create(true))?;
    file.write_all(format!("\n{}", line).as_bytes())?;

    Ok(())
}

pub fn handle_history(ctx: &mut Context) {
    if ctx.cli.current_key.code == KeyCode::Esc {
        restore_backup(ctx);
    } else if ctx.cli.current_key.code == KeyCode::Down {
        let history_len = ctx.cli.completion.list.len();
        if ctx.cli.completion.index as usize == history_len {
            print_error(ctx, "Reached start of history!");
            return;
        }

        ctx.cli.completion.index += 1;

        if ctx.cli.completion.index as usize == history_len {
            restore_backup(ctx);
            return;
        }

        let new_buf = ctx.cli.completion.list.get(ctx.cli.completion.index as usize).unwrap().to_string();
        let buf_dif = (new_buf.len() as i16) - (ctx.cli.command_buffer.len() as i16);
        ctx.cli.command_buffer = new_buf;
        print_cmd_buf(ctx, buf_dif);
    } else if ctx.cli.current_key.code == KeyCode::Up {
        if ctx.cli.last_key.code != KeyCode::Down && ctx.cli.last_key.code != KeyCode::Up {
            match find_history(ctx.cli.command_buffer.clone()) {
                Ok(history) => ctx.cli.completion.list = history,
                Err(why) => {
                    print_error(ctx, format!("Unable to get history! {}", why));
                    return;
                },
            }

            ctx.cli.completion.backup = CommandBufferBackup::new(ctx.cli.command_buffer.clone(), ctx.cli.cursor_pos.0);
            let history_len = ctx.cli.completion.list.len() as u16;
            if history_len == 0 {
                print_error(ctx, "Reached end of history!");
                return;
            }

            ctx.cli.completion.index = history_len - 1;
        } else {
            if ctx.cli.completion.index == 0 {
                print_error(ctx, "Reached end of history!");
                return;
            }

            ctx.cli.completion.index -= 1;
        }

        let new_buf = ctx.cli.completion.list.get(ctx.cli.completion.index as usize).unwrap().to_string();
        let buf_dif = (new_buf.len() as i16) - (ctx.cli.command_buffer.len() as i16);
        ctx.cli.command_buffer = new_buf;
        print_cmd_buf(ctx, buf_dif);
    }
}
