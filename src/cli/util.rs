use std::{env, fmt, io::stdout};

use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    execute,
    style::{Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    terminal::{Clear, ClearType},
    Command,
};

use crate::{command::is_valid_command, options::Options, prelude::Context};

pub fn move_cursor(ctx: &mut Context, move_size: i16) {
    if let Err(why) = if move_size >= 1 {
        execute!(&ctx.writer, MoveRight(move_size as u16))
    } else if move_size <= -1 {
        execute!(&ctx.writer, MoveLeft(move_size.abs() as u16))
    } else {
        Ok(())
    } {
        print_error(format!("Unable to move cursor! {}", why));
    }
}

pub fn print_cmd_buf(ctx: &mut Context, move_size: i16) {
    let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        MoveLeft(pos as u16),
        Clear(ClearType::UntilNewLine),
        PrintCmdBuf(ctx.command_buffer.clone(), ctx.options),
        RestorePosition,
    ) {
        print_error(format!("Unable to print command buffer! {}", why));
    }

    move_cursor(ctx, move_size);
}

pub fn print_line<T: ToString>(text: T) {
    if let Err(why) = execute!(
        stdout(),
        SavePosition,
        Print(format!("{}\n", text.to_string())),
        RestorePosition,
        MoveToNextLine(1),
    ) {
        print_error(format!("Unable to print line! {}", why));
    }
}

pub fn print_error<T: ToString>(text: T) {
    if let Err(why) = execute!(
        stdout(),
        SavePosition,
        MoveToPreviousLine(1),
        SetForegroundColor(Color::Red),
        Print("[ERROR] "),
        ResetColor,
        Print(text.to_string()),
        RestorePosition,
    ) {
        eprintln!("Unable to print error message!");
        eprintln!("Reason: {}", why);
        eprintln!("Error: {}", text.to_string());
    }
}

pub fn clear_error() {
    if let Err(why) = execute!(
        stdout(),
        SavePosition,
        MoveToPreviousLine(1),
        Clear(ClearType::CurrentLine),
        RestorePosition,
    ) {
        print_error(format!("Unable to clear error! {}", why));
    }
}

struct PrintCmdBuf<'t>(pub String, pub &'t Options);

impl<'t> Command for PrintCmdBuf<'t> {
    fn write_ansi(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let command_buffer = &self.0;
        let opts = self.1;

        let trimmed_command = command_buffer.trim();
        if trimmed_command.is_empty() || !opts.syntax_highlighting {
            return write!(writer, "{}", command_buffer);
        }

        let mut command_and_args = trimmed_command.split_whitespace();
        let command = command_and_args.next().unwrap();
        let args = format!(" {}", command_and_args.collect::<Vec<&str>>().join(" ")).reset();

        let cmd = if is_valid_command(&opts, &command) {
            command.dark_green()
        } else {
            command.dark_red()
        };

        let pos = command_buffer.find(command).unwrap_or(0);

        // TODO: Handle errors
        Print(" ".repeat(pos)).write_ansi(writer).unwrap_or_else(|_| {});
        PrintStyledContent(cmd).write_ansi(writer).unwrap_or_else(|_| {});
        PrintStyledContent(args).write_ansi(writer)
    }

    #[cfg(windows)]
    fn execute_winapi(&self, mut writer: impl FnMut() -> Result<()>) -> Result<()> {
        writer()
    }
}

pub fn format_prompt(ctx: &mut Context) {
    let mut prompt_format = ctx.options.prompt.format.clone();

    if prompt_format.contains("{PWD}") {
        let mut working_dir = env::current_dir()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("[Error]")
            .to_string();

        if ctx.options.prompt.truncate_home {
            if let Ok(home) = env::var("HOME") {
                if working_dir.starts_with(&home) {
                    working_dir = format!("~{}", working_dir[home.len()..working_dir.len()].to_string());
                }
            }
        }

        prompt_format = prompt_format.replace("{PWD}", &working_dir);
    }

    if prompt_format.contains("{HOST}") {
        let host = whoami::hostname();
        prompt_format = prompt_format.replace("{HOST}", &host);
    }

    if prompt_format.contains("{OS}") {
        let os = whoami::distro();
        prompt_format = prompt_format.replace("{OS}", &os);
    }

    if prompt_format.contains("{USER}") {
        let user = whoami::username();
        prompt_format = prompt_format.replace("{USER}", &user);
    }

    ctx.prompt = prompt_format;
}
