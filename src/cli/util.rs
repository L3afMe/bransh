use std::{env, fmt};

use crossterm::{
    cursor::{MoveLeft, MoveRight, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    execute,
    style::{Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    terminal::{Clear, ClearType},
    Command,
};

use crate::{
    command::{is_valid_command, tokenize::TokenizationError},
    options::Options,
    prelude::Context,
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
    let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt_len();
    if let Err(why) = execute!(
        &ctx.writer,
        SavePosition,
        MoveLeft(pos as u16),
        Clear(ClearType::UntilNewLine),
        PrintCmdBuf(ctx.command_buffer.clone(), &ctx.options),
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
        Print(&ctx.prompt)
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

struct PrintCmdBuf<'t>(pub String, pub &'t Options);

impl<'t> Command for PrintCmdBuf<'t> {
    fn write_ansi(&self, writer: &mut impl fmt::Write) -> fmt::Result {
        let command_buffer = &self.0;
        let opts = self.1;

        if !opts.syntax_highlighting || command_buffer.is_empty() {
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
                } else {
                    let command_str = if is_valid_command(opts, &cmd_buf) {
                        cmd_buf.clone().dark_green()
                    } else {
                        cmd_buf.clone().dark_red()
                    };

                    PrintStyledContent(command_str)
                        .write_ansi(writer)
                        .unwrap_or_else(|_| {});
                    done_cmd = true;
                }
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
                let content = if in_quote != char::default() {
                    buf_char.to_string().magenta()
                } else {
                    buf_char.to_string().reset()
                };

                PrintStyledContent(content).write_ansi(writer).unwrap_or_else(|_| {});
            }

            last_escaped = false;
            last_buf_char = buf_char;
        }

        if !done_cmd {
            let command_str = if is_valid_command(opts, &cmd_buf) {
                cmd_buf.dark_green()
            } else {
                cmd_buf.dark_red()
            };

            PrintStyledContent(command_str)
                .write_ansi(writer)
                .unwrap_or_else(|_| {});
        }

        Ok(())

        // let mut command_and_args =
        // trimmed_command.split_whitespace();
        // let command = command_and_args.next().unwrap();
        // let args = format!(" {}",
        // command_and_args.collect::<Vec<&str>>().join("
        // ")).reset();

        // let cmd = if is_valid_command(&opts, &command) {
        //     command.dark_green()
        // } else {
        //     command.dark_red()
        // };

        // let pos = command_buffer.find(command).
        // unwrap_or(0);

        // // TODO: Handle errors
        // Print(" ".repeat(pos)).write_ansi(writer).
        // unwrap_or_else(|_| {});
        // PrintStyledContent(cmd).write_ansi(writer).
        // unwrap_or_else(|_| {});
        // PrintStyledContent(args).write_ansi(writer)
    }

    #[cfg(windows)]
    fn execute_winapi(&self, mut writer: impl FnMut() -> Result<()>) -> Result<()> {
        writer()
    }
}

pub fn format_prompt(ctx: &mut Context) {
    let mut prompt_format = ctx.options.prompt.format.clone();

    if prompt_format.contains("{WD}") {
        let mut working_dir = env::current_dir()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("[Error]")
            .to_string();

        if ctx.options.prompt.truncate_home {
            if let Ok(home) = env::var("HOME") {
                if working_dir.starts_with(&home) {
                    working_dir = format!(
                        "{}{}",
                        ctx.options.prompt.truncate_home_symbol,
                        working_dir[home.len()..working_dir.len()].to_string()
                    );
                }
            }
        }

        let dir_trunc = ctx.options.prompt.truncate_directories as usize;
        if dir_trunc != 0 {
            let split: Vec<&str> = working_dir.split('/').collect();
            if split.len() > dir_trunc {
                let (_, trunc_dirs) = split.split_at(split.len() - dir_trunc);
                working_dir = format!(
                    "{}/{}",
                    ctx.options.prompt.truncate_directories_symbol,
                    trunc_dirs.join("/")
                );
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

    ctx.prompt = prompt_format;
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
