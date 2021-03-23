use std::{env, fmt};

use br_data::context::Context;
use br_parser::{TokenizationError, is_valid_command};
use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    execute,
    style::{Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    terminal::{Clear, ClearType},
    Command,
};

pub fn move_cursor(ctx: &mut Context, move_size: i16) {
    if let Err(why) = if move_size >= 1 {
        execute!(&ctx.writer, MoveRight(move_size as u16))
    } else if move_size <= -1 {
        execute!(&ctx.writer, MoveLeft(move_size.abs() as u16))
    } else {
        Ok(())
    } {
        print_error(ctx, format!("Unable to move cursor! {}", why));
    }
}

pub fn print_cmd_buf(ctx: &mut Context, move_size: i16) {
    let pos = (ctx.cli.cursor_pos.0 as usize) - ctx.cli.prompt_len();
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        MoveLeft(pos as u16),
        Clear(ClearType::UntilNewLine),
        PrintCmdBuf(ctx.cli.command_buffer.clone(), ctx),
        RestorePosition,
    ) {
        print_error(ctx, format!("Unable to print command buffer! {}", why));
    }

    move_cursor(ctx, move_size);
}

pub fn print_line<T: ToString>(ctx: &mut Context, text: T) {
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        Print(format!("{}\n", text.to_string())),
        RestorePosition,
        MoveToNextLine(1),
    ) {
        print_error(ctx, format!("Unable to print line! {}", why));
    }
}

pub fn print_prompt(ctx: &mut Context) {
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        Print("\n".to_string()),
        RestorePosition,
        MoveToNextLine(1),
        Print(&ctx.cli.prompt)
    ) {
        print_error(ctx, format!("Unable to print prompt! {}", why));
    }
}

pub fn print_error<T: ToString>(ctx: &mut Context, text: T) {
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        MoveToPreviousLine(1),
        SetForegroundColor(Color::Red),
        Print("[ERROR] "),
        ResetColor,
        Print(text.to_string()),
        Clear(ClearType::UntilNewLine),
        RestorePosition,
    ) {
        eprintln!("Unable to print error message!");
        eprintln!("Reason: {}", why);
        eprintln!("Error: {}", text.to_string());
    }
}

pub fn clear_error(ctx: &mut Context) {
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        MoveToPreviousLine(1),
        Clear(ClearType::CurrentLine),
        RestorePosition,
    ) {
        print_error(ctx, format!("Unable to clear error! {}", why));
    }
}

struct PrintCmdBuf<'t>(pub String, pub &'t Context);

impl<'t> Command for PrintCmdBuf<'t> {
    fn write_ansi(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let command_buffer = &self.0;
        let ctx = self.1;

        if !ctx.get_variable("SYN_HIGHLIGHTING", true, false) || command_buffer.is_empty() {
            return write!(writer, "{}", command_buffer);
        }

        let mut buffer = command_buffer.trim_start();

        // Store trim size so syntax positions are correct
        let left_trim_size = command_buffer.len() - buffer.len();
        buffer = buffer.trim_end();
        write!(writer, "{}", " ".repeat(left_trim_size))?;

        if buffer.is_empty() {
            return Ok(());
        }

        let mut in_quote = char::default();

        let mut last_buf_char = char::default();
        let mut last_escaped = false;

        let mut done_cmd = false;
        let mut cmd_buf = String::new();

        for buf_char in buffer.chars() {
            if !done_cmd {
                if (cmd_buf.is_empty() || buf_char.is_alphanumeric()) || buf_char == '_' || buf_char == '-' {
                    if buf_char == ' ' {
                        Print(" ").write_ansi(writer).unwrap_or_else(|_| {});
                    } else {
                        cmd_buf.push(buf_char);
                    }
                    continue;
                }

                let command_str = if is_valid_command(&cmd_buf, ctx) {
                    cmd_buf.clone().dark_green()
                } else {
                    cmd_buf.clone().dark_red()
                };

                PrintStyledContent(command_str)
                    .write_ansi(writer)
                    .unwrap_or_else(|_| {});
                done_cmd = true;
            }

            if last_buf_char == '\\' && !last_escaped {
                last_escaped = true;
                last_buf_char = buf_char;

                PrintStyledContent(format!("\\{}", buf_char).dark_blue())
                    .write_ansi(writer)
                    .unwrap_or_else(|_| {});
                continue;
            } else if buf_char == '\'' || buf_char == '"' {
                if in_quote == char::default() {
                    in_quote = buf_char;
                } else if in_quote == buf_char {
                    in_quote = char::default();
                };

                PrintStyledContent(buf_char.to_string().magenta())
                    .write_ansi(writer)
                    .unwrap_or_else(|_| {});
            } else if (buf_char == ';' || buf_char == '|') && in_quote == char::default() {
                done_cmd = false;
                cmd_buf = String::new();

                PrintStyledContent(buf_char.to_string().reset())
                    .write_ansi(writer)
                    .unwrap_or_else(|_| {});
            } else if buf_char != '\\' {
                let content = if in_quote == char::default() {
                    buf_char.to_string().reset()
                } else {
                    buf_char.to_string().magenta()
                };

                PrintStyledContent(content).write_ansi(writer).unwrap_or_else(|_| {});
            }

            last_escaped = false;
            last_buf_char = buf_char;
        }

        if !done_cmd {
            let command_str = if is_valid_command(&cmd_buf, ctx) {
                cmd_buf.dark_green()
            } else {
                cmd_buf.dark_red()
            };

            PrintStyledContent(command_str)
                .write_ansi(writer)
                .unwrap_or_else(|_| {});
        }

        Ok(())
    }

    #[cfg(windows)]
    fn execute_winapi(&self, mut writer: impl FnMut() -> Result<()>) -> Result<()> {
        writer()
    }
}

pub fn format_prompt(ctx: &mut Context) {
    let mut prompt_format = ctx.get_variable("PROMPT", String::from("{WD} | "), false);

    if prompt_format.contains("{WD}") {
        let mut working_dir = env::current_dir()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("[Error]")
            .to_string();

        let home_trunc = ctx.get_variable("P_HOME_TRUNC", true, false);
        if home_trunc {
            if let Ok(home) = env::var("HOME") {
                if working_dir.starts_with(&home) {
                    let home_trunc_char = ctx.get_variable("P_HOME_CHAR", String::from("~"), false);
                    working_dir = format!(
                        "{}{}",
                        home_trunc_char,
                        working_dir[home.len()..working_dir.len()].to_string()
                    );
                }
            }
        }

        let dir_trunc = ctx.get_variable("P_DIR_TRUNC", 2, false);
        if dir_trunc != 0 {
            let split: Vec<&str> = working_dir.split('/').collect();
            if split.len() > dir_trunc {
                let dir_trunc_char = ctx.get_variable("P_DIR_CHAR", String::from("…"), false);
                let (_, trunc_dirs) = split.split_at(split.len() - dir_trunc);
                working_dir = format!("{}/{}", dir_trunc_char, trunc_dirs.join("/"));
            }
        }

        prompt_format = prompt_format.replace("{WD}", &working_dir);
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

    ctx.cli.prompt = prompt_format;
}

pub fn print_tokenization_error(ctx: &mut Context, error: TokenizationError) {
    let error_str = match error {
        TokenizationError::UnterminateQuote(pos, chr) => format!("Unterminated string ({}) at pos {}", chr, pos),
        TokenizationError::InvalidEscape(pos, chr) => format!("Invalid escape code (\\{}) at pos {}", chr, pos),
        TokenizationError::UnexpectedValue(pos, val_exp, val_got) => {
            format!("Unexpected value at pos {}, expected {}, got {}", pos, val_exp, val_got)
        },
    };

    print_error(ctx, error_str);
}

pub fn restore_backup(ctx: &mut Context) {
    ctx.cli.command_buffer = ctx.cli.completion.backup.buffer.clone();
    let new_pos = (ctx.cli.completion.backup.cursor as i16) - (ctx.cli.cursor_pos.0 as i16);
    print_cmd_buf(ctx, new_pos);
}
