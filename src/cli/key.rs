use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    cli::{
        tabcomp::{clear_tab, handle_tab},
        util::{move_cursor, print_cmd_buf, print_error},
    },
    prelude::Context,
};

pub fn handle_key(ctx: &mut Context) -> bool {
    match ctx.current_key.code {
        KeyCode::Enter => return false,
        KeyCode::Up => {
            print_error("Test error on up press, history not yet implemented");
        },
        
        // Command buffer manipulation
        KeyCode::Char(_) => handle_char(ctx),
        KeyCode::Backspace => handle_backspace(ctx),
        KeyCode::Delete => handle_delete(ctx),

        // Tab completion
        KeyCode::Tab => handle_tab(ctx),
        KeyCode::Esc => clear_tab(ctx),

        // Movement
        KeyCode::Home => move_sol(ctx),
        KeyCode::End => move_eol(ctx),
        KeyCode::Left => move_left(ctx),
        KeyCode::Right => move_right(ctx),
        _ => {},
    }

    true
}

fn handle_char(ctx: &mut Context) {
    let pressed_key = if let KeyCode::Char(pressed_key) = ctx.current_key.code {
        pressed_key
    } else {
        return;
    };

    let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();

    if pressed_key == 'c' && ctx.current_key.modifiers == KeyModifiers::CONTROL {
        ctx.command_buffer = String::new();
        print_cmd_buf(ctx, -(pos as i16));

        return;
    }

    ctx.command_buffer.insert(pos, pressed_key);
    print_cmd_buf(ctx, 1);
}

fn move_sol(ctx: &mut Context) {
    let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();

    move_cursor(ctx, -(pos as i16));
}

fn move_eol(ctx: &mut Context) {
    let pos = ctx.prompt.len() + ctx.command_buffer.len() - (ctx.cursor_pos.0 as usize);

    move_cursor(ctx, pos as i16);
}

fn move_left(ctx: &mut Context) {
    if (ctx.cursor_pos.0 as usize) > ctx.prompt.len() {
        let mut move_size = 1;
        if let KeyModifiers::CONTROL = ctx.current_key.modifiers {
            let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
            let (split, _) = ctx.command_buffer.split_at(pos.min(ctx.command_buffer.len()));

            move_size = split.len() - split.rfind(' ').unwrap_or_else(|| split.len());
        }

        move_cursor(ctx, -(move_size as i16));
    }
}

fn move_right(ctx: &mut Context) {
    if (ctx.cursor_pos.0 as usize) < ctx.prompt.len() + ctx.command_buffer.len() {
        let mut move_size = 1;
        if let KeyModifiers::CONTROL = ctx.current_key.modifiers {
            let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
            let (_, split) = ctx.command_buffer.split_at(pos.min(ctx.command_buffer.len()));

            // Add 1 to go to start of next word
            move_size = split.find(' ').unwrap_or_else(|| split.len() - 1) + 1;
        }

        move_cursor(ctx, move_size as i16);
    }
}

fn handle_backspace(ctx: &mut Context) {
    let x = ctx.cursor_pos.0 as usize;
    if x > ctx.prompt.len() {
        let pos = x - ctx.prompt.len() - 1;
        ctx.command_buffer.remove(pos);
        print_cmd_buf(ctx, -1);
    }
}

fn handle_delete(ctx: &mut Context) {
    let x = ctx.cursor_pos.0 as usize;
    if x > ctx.prompt.len() {
        let pos = x - ctx.prompt.len();
        ctx.command_buffer.remove(pos);
        print_cmd_buf(ctx, 0);
    }
}
