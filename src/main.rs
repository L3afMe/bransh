mod cli;
mod command;
mod prelude;
mod script;
mod options;

fn main() {
    let opts = options::parse();

    if opts.version {
        println!("Bransh v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(command) = opts.command {
        command::execute_once(command);
        return;
    }

    if let Err(why) = cli::run_term(opts) {
        eprintln!("Error occured while running terminal! {}", why);
    }
}
