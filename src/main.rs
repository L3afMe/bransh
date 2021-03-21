pub mod command;
pub mod options;
pub mod prelude;
pub mod cli;

fn main() {
    let opts = options::load_options();

    if let Err(why) = cli::run_term(opts) {
        eprintln!("Error occured while running terminal! {}", why);
    } 
}
