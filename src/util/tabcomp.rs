use std::ffi::OsString;

use crossterm::event::KeyCode;

use crate::{command, prelude::Context, print_error};

pub struct TabContext {
    pub index:          i16,
    pub list:           Vec<OsString>,
    pub cmd_buf_backup: String,
}

impl TabContext {
    pub fn new() -> Self {
        Self {
            index:          0,
            list:           Vec::new(),
            cmd_buf_backup: String::new(),
        }
    }
}

pub fn handle_tab(ctx: &mut Context) {
        let trimmed_command = ctx.command_buffer.trim_start();
        if trimmed_command.contains(" ") {
            // TODO: Arg tab
            // completion
        } else {
            if ctx.last_key.code != KeyCode::Tab {
                ctx.tab.list = command::get_valid_commands(&ctx.options)
                    .into_iter()
                    .filter(|cmd| cmd.to_str().unwrap_or("").starts_with(trimmed_command))
                    .collect();
                ctx.tab.index = 0;
                ctx.tab.cmd_buf_backup = ctx.command_buffer.clone();
            } else {
                ctx.tab.index += 1;
            }

            if ctx.tab.index == ctx.tab.list.len() as i16 {
                ctx.command_buffer = ctx.tab.cmd_buf_backup.clone();
                ctx.tab.index = -1;
            } else {
                match ctx.tab.list.get(ctx.tab.index as usize) {
                    Some(tab_val) => ctx.command_buffer = tab_val.to_str().unwrap_or(&ctx.command_buffer).to_string(),
                    None => print_error("Unable to get tab completion value"),
                }
            }
        }
    }
