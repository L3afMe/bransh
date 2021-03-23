use br_data::context::Context;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    history::handle_history,
    tabcomp::{clear_tab, handle_tab},
    util::{move_cursor, print_cmd_buf},
};

pub fn handle_key(ctx: &mut Context) -> bool {
    match ctx.cli.current_key.code {
        KeyCode::Enter => return false,

        // History
        KeyCode::Up | KeyCode::Down => handle_history(ctx),

        // Command buffer manipulation
        KeyCode::Char(_) => handle_char(ctx),
        KeyCode::Backspace => handle_backspace(ctx),
        KeyCode::Delete => handle_delete(ctx),

        // Tab completion
        KeyCode::Tab => handle_tab(ctx),
        KeyCode::Esc => handle_esc(ctx),

        // Movement
        KeyCode::Home => move_sol(ctx),
        KeyCode::End => move_eol(ctx),
        KeyCode::Left => move_left(ctx),
        KeyCode::Right => move_right(ctx),
        _ => {},
    }

    true
}

fn handle_esc(ctx: &mut Context) {
    if ctx.cli.last_key.code == KeyCode::Tab {
        clear_tab(ctx);
    } else if ctx.cli.last_key.code == KeyCode::Up || ctx.cli.last_key.code == KeyCode::Down {
        handle_history(ctx);
    }
}

fn handle_char(ctx: &mut Context) {
    let pressed_key = if let KeyCode::Char(pressed_key) = ctx.cli.current_key.code {
        pressed_key
    } else {
        return;
    };

    let pos = (ctx.cli.cursor_pos.0 as usize) - ctx.cli.prompt_len();

    if pressed_key == 'c' && ctx.cli.current_key.modifiers == KeyModifiers::CONTROL {
        ctx.cli.command_buffer = String::new();
        print_cmd_buf(ctx, -(pos as i16));

        return;
    }

    ctx.cli.command_buffer.insert(pos, pressed_key);
    print_cmd_buf(ctx, 1);
}

fn move_sol(ctx: &mut Context) {
    let pos = (ctx.cli.cursor_pos.0 as usize) - ctx.cli.prompt_len();

    move_cursor(ctx, -(pos as i16));
}

fn move_eol(ctx: &mut Context) {
    let pos = ctx.cli.prompt_len() + ctx.cli.command_buffer.len() - (ctx.cli.cursor_pos.0 as usize);

    move_cursor(ctx, pos as i16);
}

fn move_left(ctx: &mut Context) {
    if (ctx.cli.cursor_pos.0 as usize) > ctx.cli.prompt_len() {
        let move_size = if ctx.cli.current_key.modifiers == KeyModifiers::CONTROL {
            let pos = (ctx.cli.cursor_pos.0 as usize) - ctx.cli.prompt_len();
            let t = ctx.cli.command_buffer.clone();

            let (split, _) = t.split_at(pos.min(ctx.cli.command_buffer.len()));
            let split_trim = split.trim_end();
            let trim_len = split.len() - split_trim.len();

            match split_trim.rfind(' ') {
                Some(pos) => split_trim.len() - pos - 1 + trim_len,
                None => split.len(),
            }
        } else {
            1
        };

        move_cursor(ctx, -(move_size as i16));
    }
}

fn move_right(ctx: &mut Context) {
    if (ctx.cli.cursor_pos.0 as usize) < ctx.cli.prompt_len() + ctx.cli.command_buffer.len() {
        let move_size = if ctx.cli.current_key.modifiers == KeyModifiers::CONTROL {
            let pos = (ctx.cli.cursor_pos.0 as usize) - ctx.cli.prompt_len();
            let (_, split) = ctx.cli.command_buffer.split_at(pos.min(ctx.cli.command_buffer.len()));

            split.find(' ').unwrap_or_else(|| split.len() - 1) + 1
        } else {
            1
        };

        move_cursor(ctx, move_size as i16);
    }
}

fn handle_backspace(ctx: &mut Context) {
    let x = ctx.cli.cursor_pos.0 as usize;
    if x > ctx.cli.prompt_len() {
        let pos = x - ctx.cli.prompt_len() - 1;
        ctx.cli.command_buffer.remove(pos);
        print_cmd_buf(ctx, -1);
    }
}

fn handle_delete(ctx: &mut Context) {
    let x = ctx.cli.cursor_pos.0 as usize;
    if x > ctx.cli.prompt_len() {
        let pos = x - ctx.cli.prompt_len();
        ctx.cli.command_buffer.remove(pos);
        print_cmd_buf(ctx, 0);
    }
}
