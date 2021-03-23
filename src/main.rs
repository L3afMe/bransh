fn main() {
    let opts = br_data::options::Options::parse();

    if opts.version {
        println!("Bransh v{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(command) = opts.command {
        br_executer::execute_once(command);
        return;
    }

    if let Err(why) = br_cli::run_term(opts) {
        eprintln!("Error occured while running terminal! {}", why);
    }
}
