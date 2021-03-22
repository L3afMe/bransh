mod cli;
mod command;
mod prelude;
mod script;

fn main() {
    if let Err(why) = cli::run_term() {
        eprintln!("Error occured while running terminal! {}", why);
    }
}
