use br_command::get_tab_completion;
use br_data::context::{CommandBufferBackup, Context};
use br_parser::get_valid_commands;
use crossterm::event::KeyCode;

use crate::util::{print_cmd_buf, print_error, restore_backup};

pub fn handle_tab(ctx: &mut Context) {
    let old_buf = ctx.cli.command_buffer.clone();

    let trimmed_command = old_buf.trim_start();
    let trim_size = old_buf.len() - trimmed_command.len();

    let (trimmed_command, after_cursor) =
        trimmed_command.split_at(ctx.cli.cursor_pos.0 as usize - ctx.cli.prompt_len());

    if ctx.cli.last_key.code == KeyCode::Tab {
        if ctx.cli.completion.index == ctx.cli.completion.list.len() as u16 {
            ctx.cli.completion.index = 0
        } else {
            ctx.cli.completion.index += 1;
        }
    } else {
        ctx.cli.completion.index = 0;
        ctx.cli.completion.backup = CommandBufferBackup::new(ctx.cli.command_buffer.clone(), ctx.cli.cursor_pos.0);

        // Get tabcomp list
        ctx.cli.completion.list = if trimmed_command.contains(' ') {
            let mut args: Vec<String> = trimmed_command.split(' ').map(|arg| arg.to_string()).collect();
            let cmd = args.remove(0);
            get_tab_completion(cmd, args, ctx)
        } else {
            get_valid_commands(ctx)
                .into_iter()
                .filter(|cmd| cmd.starts_with(trimmed_command))
                .collect()
        };
    }

    fn do_comp(
        new_arg: &str,
        trimmed_command: &str,
        trim_size: usize,
        old_buf: &str,
        after_cursor: &str,
        ctx: &mut Context,
    ) {
        let dif = if trimmed_command.contains(' ') {
            // Arg completion
            let right_space = trimmed_command.rfind(' ').unwrap();
            let (before_arg, _arg) = trimmed_command.split_at(right_space);

            let new_buf = format!("{}{} {}{}", " ".repeat(trim_size), before_arg, new_arg, after_cursor);
            ctx.cli.command_buffer = new_buf.clone();

            (new_buf.len() as i16) - (old_buf.len() as i16)
        } else {
            // Command completion
            let new_buf = format!("{}{}{}", " ".repeat(trim_size), new_arg, after_cursor);
            ctx.cli.command_buffer = new_buf.clone();

            (new_buf.len() as i16) - (old_buf.len() as i16)
        };

        print_cmd_buf(ctx, dif);
    }

    if ctx.cli.completion.list.len() == 1 {
        let mut new_arg = ctx.cli.completion.list[0].clone();
        new_arg.push_str(&ctx.cli.completion.delim);
        do_comp(&new_arg, trimmed_command, trim_size, &old_buf, after_cursor, ctx);

        ctx.cli.current_key.code = KeyCode::Null;
        return;
    }

    if ctx.cli.completion.index == ctx.cli.completion.list.len() as u16 {
        restore_backup(ctx);
    } else {
        match ctx.cli.completion.list.clone().get(ctx.cli.completion.index as usize) {
            Some(new_arg) => do_comp(&new_arg, trimmed_command, trim_size, &old_buf, after_cursor, ctx),
            None => {
                print_error(ctx, "Unable to get tab completion value");
            },
        }
    };
}
