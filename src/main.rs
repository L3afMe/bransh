use std::{env, fmt, io::stdout, path::PathBuf};

use command::execute;
use crossterm::{
    cursor::{position, MoveLeft, MoveRight, MoveToNextLine, MoveToPreviousLine, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Colorize, Print, PrintStyledContent, ResetColor, SetForegroundColor, Styler},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Command, Result,
};
use prelude::handle_tab;

pub mod command;
pub mod options;
pub mod prelude;
pub mod util;

fn main() {
    if let Err(why) = enable_raw_mode() {
        panic!("Unable to enable raw mode! {}", why);
    }

    let opts = options::load_options();

    if let Err(why) = run_term(opts) {
        eprintln!("Error occured while running terminal! {}", why);
    }

    if let Err(why) = disable_raw_mode() {
        panic!("Unable to disable raw mode! Run 'reset' to manually disable! {}", why);
    }

    println!("\nSee you later!");
}

fn run_term(opts: options::Options) -> Result<()> {
    let mut stdout = stdout();

    let mut ctx = prelude::Context::new(&opts);

    // Add later on for custom prompt or something idk
    #[allow(unused_variables, unused_mut)]
    let mut last_exit_code = 0;

    'mainloop: loop {
        let mut working_dir = env::current_dir()
            .unwrap_or(PathBuf::new())
            .to_str()
            .unwrap_or("[Error]")
            .to_string();

        if let Ok(home) = env::var("HOME") {
            if working_dir.starts_with(&home) {
                working_dir = format!("~{}", working_dir[home.len()..working_dir.len()].to_string());
            }
        }

        ctx.command_buffer = String::new();
        ctx.prompt = format!("{} | ", working_dir);

        // Mandatory newline to ensure there is line for
        // errors to go on
        if let Err(why) = execute!(
            stdout,
            SavePosition,
            Print("\n".to_string()),
            RestorePosition,
            MoveToNextLine(1),
            Print(&ctx.prompt)
        ) {
            print_error(format!("Unable to print prompt! {}", why));
        }

        'keyloop: loop {
            let key = match read() {
                Ok(event) => event,
                Err(why) => {
                    print_error(format!("Unable to capture event! {}", why));
                    continue;
                },
            };

            clear_error();

            #[allow(unused_variables)]
            match position() {
                Ok(pos) => ctx.cursor_pos = pos,
                Err(why) => {
                    print_error(format!("Unable to get cursor position! {}", why));
                    continue;
                },
            };

            match key {
                Event::Key(event) => {
                    ctx.current_key = event;
                    if !handle_key(&mut ctx) {
                        break 'keyloop;
                    }
                    ctx.last_key = event;
                },
                _ => {},
            }
        }

        if let Err(why) = execute!(
            stdout,
            SavePosition,
            Print("\n".to_string()),
            RestorePosition,
            MoveToNextLine(1),
        ) {
            print_error(format!("Unable to process newline! {}", why));
        };

        let trimmed_command = ctx.command_buffer.trim();
        if trimmed_command.is_empty() {
            continue;
        }

        let mut command_and_args = trimmed_command.split_whitespace();
        let command = command_and_args.next().unwrap();
        let args = command_and_args;

        match command {
            "exit" => break 'mainloop,
            _ => {
                let response = execute(&opts, command.to_string(), args);

                if response != 0 {
                    // TODO: Print error if
                    // verbose or something
                }
            },
        }
    }

    Ok(())
}

fn handle_key(ctx: &mut prelude::Context) -> bool {
    match ctx.current_key.code {
        KeyCode::Tab => {
            handle_tab(ctx);
            let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
            if let Err(why) = execute!(
                &ctx.writer,
                MoveLeft(pos as u16),
                Clear(ClearType::UntilNewLine),
                PrintCmdBuf(ctx.command_buffer.clone(), ctx.options),
                MoveLeft(1),
            ) {
                print_error(format!("Unable to process tab completion! {}", why));
            }
        },
        KeyCode::Esc => {
            ctx.command_buffer = ctx.tab.cmd_buf_backup.clone();

            let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
            if let Err(why) = execute!(
                &ctx.writer,
                MoveLeft(pos as u16),
                Clear(ClearType::UntilNewLine),
                PrintCmdBuf(ctx.command_buffer.clone(), ctx.options),
                MoveLeft(1),
            ) {
                print_error(format!("Unable to process escape! {}", why));
            }
        },
        KeyCode::Up => {
            print_error("Test error on up press, history not yet implemented");
        },
        KeyCode::Char(pressed_key) => {
            if pressed_key == 'c' && ctx.current_key.modifiers == KeyModifiers::CONTROL {
                ctx.command_buffer = String::new();

                if let Err(why) = execute!(
                    &ctx.writer,
                    SavePosition,
                    MoveLeft(ctx.cursor_pos.0),
                    Clear(ClearType::CurrentLine),
                    Print(&ctx.prompt),
                ) {
                    print_error(format!("Unable to process key press! {}", why));
                    return true;
                }

                return true;
            }

            let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
            ctx.command_buffer.insert(pos, pressed_key);

            if let Err(why) = execute!(
                &ctx.writer,
                SavePosition,
                MoveLeft(pos as u16),
                PrintCmdBuf(ctx.command_buffer.clone(), ctx.options),
                RestorePosition,
                MoveRight(1)
            ) {
                print_error(format!("Unable to process key press! {}", why));
            }
        },
        KeyCode::Enter => return false,
        KeyCode::Left => {
            if (ctx.cursor_pos.0 as usize) > ctx.prompt.len() {
                if let Err(why) = execute!(&ctx.writer, MoveLeft(1)) {
                    print_error(format!("Unable to process key press! {}", why));
                }
            }
        },
        KeyCode::Right => {
            if (ctx.cursor_pos.0 as usize) < ctx.prompt.len() + ctx.command_buffer.len() {
                if let Err(why) = execute!(&ctx.writer, MoveRight(1)) {
                    print_error(format!("Unable to process key press! {}", why));
                }
            }
        },
        KeyCode::Backspace => {
            let cursor_x_u = ctx.cursor_pos.0 as usize;
            if cursor_x_u > ctx.prompt.len() {
                let pos = (ctx.cursor_pos.0 as usize) - ctx.prompt.len();
                ctx.command_buffer.remove(pos - 1);

                if let Err(why) = execute!(
                    &ctx.writer,
                    SavePosition,
                    MoveLeft(pos as u16),
                    Clear(ClearType::UntilNewLine),
                    PrintCmdBuf(ctx.command_buffer.clone(), ctx.options),
                    RestorePosition,
                    MoveLeft(1),
                ) {
                    print_error(format!("Unable to process backspace! {}", why));
                }
            }
        },
        _ => {},
    }

    true
}

fn print_line<T: ToString>(text: T) {
    if let Err(why) = execute!(stdout(), Print(format!("{}\n", text.to_string())), MoveToNextLine(1)) {
        print_error(format!("Unable to print line! {}", why));
    }
}

fn print_error<T: ToString>(text: T) {
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

fn clear_error() {
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

pub struct PrintCmdBuf<'t>(pub String, pub &'t options::Options);

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

        let cmd = if command::is_valid_command(&opts, &command) {
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
