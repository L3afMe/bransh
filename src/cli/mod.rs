use crossterm::{
    cursor::{position, MoveToNextLine, RestorePosition, SavePosition},
    event::{read, Event},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::{command::execute, options::Options, prelude::Context};

pub mod key;
pub mod tabcomp;
pub mod util;

use key::handle_key;
use util::{clear_error, format_prompt, print_error, print_line};

pub fn run_term(opts: Options) -> Result<()> {
    if let Err(why) = enable_raw_mode() {
        panic!("Unable to enable raw mode! {}", why);
    }

    let mut ctx = Context::new(&opts);

    loop {
        ctx.command_buffer = String::new();
        format_prompt(&mut ctx);

        // Mandatory newline to ensure there is line for
        // errors to go on
        if let Err(why) = execute!(
            &ctx.writer,
            SavePosition,
            Print("\n".to_string()),
            RestorePosition,
            MoveToNextLine(1),
            Print(&ctx.prompt)
        ) {
            print_error(format!("Unable to print prompt! {}", why));
        }

        loop {
            let event_wrapped = read();
            if let Err(why) = event_wrapped {
                print_error(format!("Unable to capture event! {}", why));
                continue;
            };
            let event = event_wrapped.unwrap();

            match position() {
                Ok(pos) => ctx.cursor_pos = pos,
                Err(why) => {
                    print_error(format!("Unable to get cursor position! {}", why));
                    continue;
                },
            };

            if let Event::Key(key) = event {
                clear_error();
                ctx.last_key = ctx.current_key;
                ctx.current_key = key;
                if !handle_key(&mut ctx) {
                    break;
                }
            }
        }

        print_line("");

        // Disable raw mode so commands function normally
        if let Err(why) = disable_raw_mode() {
            print_line(format!("Unable to disable raw mode! {}", why));
        }

        let response = execute(&mut ctx);
        if response.is_none() {
            break;
        }

        // Re-enable raw mode so terminal works properly
        if let Err(why) = enable_raw_mode() {
            panic!("Unable to re-enable raw mode! {}", why);
        }

        let exit_code = response.unwrap(); 
        if exit_code != 0 {
            // TODO: Something with error code
        }
    }

    if let Err(why) = disable_raw_mode() {
        panic!("Unable to disable raw mode! Run 'reset' to manually disable! {}", why);
    }

    println!("\nSee you later!");

    Ok(())
}
