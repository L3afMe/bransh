use br_command::load_builtins;
use br_data::{context::Context, options::Options};
use br_executer::execute;
use br_parser::parse_command;
use br_script::load_rc;
use crossterm::{
    cursor::position,
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::{
    key::handle_key,
    util::{clear_error, format_prompt, print_error, print_line, print_prompt, print_tokenization_error},
};

mod history;
mod key;
mod tabcomp;
mod util;

pub fn run_term(opts: Options) -> Result<()> {
    // Set dummy handler so that ctrl-c doesn't terminate
    // cli when running commands as raw mode is disabled.
    // Print line so that the last output line doesn't get
    // cleared on clear_error()
    ctrlc::set_handler(move || {
        println!(" ");
    })
    .expect("Unable to setup ctrl-c handler");

    let mut ctx = Context::default();
    load_builtins(&mut ctx);

    if !opts.norc {
        load_rc(&mut ctx);
    }

    if let Err(why) = history::init_history() {
        println!("Unable to initialise history file! {}", why);
    }

    if let Err(why) = enable_raw_mode() {
        panic!("Unable to enable raw mode! {}", why);
    }

    loop {
        ctx.cli.command_buffer = String::new();
        format_prompt(&mut ctx);
        print_prompt(&mut ctx);

        loop {
            let event_wrapped = read();
            if let Err(why) = event_wrapped {
                print_error(&mut ctx, format!("Unable to capture event! {}", why));
                continue;
            };
            let event = event_wrapped.unwrap();

            match position() {
                Ok(pos) => ctx.cli.cursor_pos = pos,
                Err(why) => {
                    print_error(&mut ctx, format!("Unable to get cursor position! {}", why));
                    continue;
                },
            };

            if let Event::Key(key) = event {
                ctx.cli.last_key = ctx.cli.current_key;
                ctx.cli.current_key = key;
                if !handle_key(&mut ctx) {
                    break;
                }

                let buffer = ctx.cli.command_buffer.clone();
                if !buffer.is_empty() {
                    if let Err(why) = parse_command(buffer, &ctx) {
                        print_tokenization_error(&mut ctx, why);
                        continue;
                    }
                }

                clear_error(&mut ctx);
            }
        }

        print_line(&mut ctx, "");

        // Disable raw mode so commands function normally
        if let Err(why) = disable_raw_mode() {
            print_line(&mut ctx, format!("Unable to disable raw mode! {}", why));
        }

        if let Err(why) = history::add_history(ctx.cli.command_buffer.clone()) {
            print_line(&mut ctx, format!("Unable to save command to history! {}", why))
        };

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
