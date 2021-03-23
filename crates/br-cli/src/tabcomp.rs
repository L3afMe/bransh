use br_data::context::{CommandBufferBackup, Context};
use br_parser::get_valid_commands;
use crossterm::event::KeyCode;

use crate::util::{print_cmd_buf, print_error, restore_backup};

pub fn handle_tab(ctx: &mut Context) {
    let old_buf = ctx.cli.command_buffer.clone();
    let trimmed_command = old_buf.trim_start();
    let (trimmed_command, _) = trimmed_command.split_at(ctx.cli.cursor_pos.0 as usize - ctx.cli.prompt_len());
    if trimmed_command.contains(' ') {
        // TODO: Arg tab
        // completion
    } else {
        if ctx.cli.last_key.code == KeyCode::Tab {
            if ctx.cli.completion.index == ctx.cli.completion.list.len() as u16 {
                ctx.cli.completion.index = 0
            } else {
                ctx.cli.completion.index += 1;
            }
        } else {
            ctx.cli.completion.index = 0;
            ctx.cli.completion.backup = CommandBufferBackup::new(ctx.cli.command_buffer.clone(), ctx.cli.cursor_pos.0);
            ctx.cli.completion.list = get_valid_commands(ctx)
                .into_iter()
                .filter(|cmd| cmd.starts_with(trimmed_command))
                .collect();
        }

        if ctx.cli.completion.index == ctx.cli.completion.list.len() as u16 {
            clear_tab(ctx);
        } else {
            match ctx.cli.completion.list.get(ctx.cli.completion.index as usize) {
                Some(new_cmd) => {
                    let new_buf = ctx.cli.command_buffer.clone().replace(trimmed_command, new_cmd);
                    ctx.cli.command_buffer = new_buf.clone();

                    let dif = (new_buf.len() as i16) - (old_buf.len() as i16);
                    print_cmd_buf(ctx, dif);
                },
                None => {
                    print_error(ctx, "Unable to get tab completion value");
                },
            }
        };
    }
}

pub fn clear_tab(ctx: &mut Context) {
    restore_backup(ctx);
}
