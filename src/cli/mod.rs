use crossterm::{
    cursor::position,
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

use crate::{
    cli::{
        key::handle_key,
        util::{clear_error, format_prompt, print_error, print_line, print_prompt, print_tokenization_error},
    },
    command::{execute, tokenize::tokenize_command},
    options::Options,
    prelude::Context,
    script::load_rc,
};

pub mod history;
pub mod key;
pub mod tabcomp;
pub mod util;

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

    if opts.norc {
        load_rc(&mut ctx);
    }

    if let Err(why) = history::init_history() {
        println!("Unable to initialise history file! {}", why);
    }

    if let Err(why) = enable_raw_mode() {
        panic!("Unable to enable raw mode! {}", why);
    }

    loop {
        ctx.command_buffer = String::new();
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
                Ok(pos) => ctx.cursor_pos = pos,
                Err(why) => {
                    print_error(&mut ctx, format!("Unable to get cursor position! {}", why));
                    continue;
                },
            };

            if let Event::Key(key) = event {
                ctx.last_key = ctx.current_key;
                ctx.current_key = key;
                if !handle_key(&mut ctx) {
                    break;
                }

                if !ctx.command_buffer.is_empty() {
                    if let Err(why) = tokenize_command(ctx.command_buffer.clone()) {
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

        if let Err(why) = history::add_history(ctx.command_buffer.clone()) {
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
