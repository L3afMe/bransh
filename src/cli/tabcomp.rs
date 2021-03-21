use crossterm::event::KeyCode;

use crate::{
    cli::util::{print_cmd_buf, print_error},
    command::get_valid_commands,
    prelude::Context,
};

#[derive(Debug, Default)]
struct TabBackup {
    command_buffer: String,
    cursor_pos:     u16,
}

impl TabBackup {
    fn new(command_buffer: String, cursor_pos: u16) -> Self {
        Self {
            command_buffer,
            cursor_pos,
        }
    }
}

#[derive(Debug, Default)]
pub struct TabContext {
    pub index: i16,
    pub list:  Vec<String>,
    backup:    TabBackup,
}

pub fn handle_tab(ctx: &mut Context) {
    let old_buf = ctx.command_buffer.clone();
    let trimmed_command = old_buf.trim_start();
    let (trimmed_command, _) = trimmed_command.split_at(ctx.cursor_pos.0 as usize - ctx.prompt.len());
    if trimmed_command.contains(' ') {
        // TODO: Arg tab
        // completion
    } else {
        if ctx.last_key.code != KeyCode::Tab {
            ctx.tab.index = 0;
            ctx.tab.backup = TabBackup::new(ctx.command_buffer.clone(), ctx.cursor_pos.0);
            ctx.tab.list = get_valid_commands(&ctx.options)
                .into_iter()
                .filter(|cmd| cmd.starts_with(trimmed_command))
                .collect();
        } else {
            ctx.tab.index += 1;
        }

        if ctx.tab.index == ctx.tab.list.len() as i16 {
            clear_tab(ctx);

            // Set to negative 1 as the last key is still tab
            // and next tab will increment it by 1 which will
            // set it to 0, could alternatively set last key 
            // to escape but this will require the tab list to
            // be refreshed again and will cause more delay
            ctx.tab.index = -1;
        } else {
            match ctx.tab.list.get(ctx.tab.index as usize) {
                Some(new_cmd) => {
                    let new_buf = ctx.command_buffer.clone().replace(trimmed_command, &new_cmd);
                    ctx.command_buffer = new_buf.clone();

                    let dif = (new_buf.len() as i16) - (old_buf.len() as i16);
                    print_cmd_buf(ctx, dif);
                },
                None => {
                    print_error("Unable to get tab completion value");
                },
            }
        };
    }
}

pub fn clear_tab(ctx: &mut Context) {
    ctx.command_buffer = ctx.tab.backup.command_buffer.clone();
    let new_pos = (ctx.tab.backup.cursor_pos as i16) - (ctx.cursor_pos.0 as i16);
    print_cmd_buf(ctx, new_pos);
}
