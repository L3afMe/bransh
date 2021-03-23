use std::env;

use crate::prelude::Context;

pub fn execute(args: Vec<String>, ctx: &mut Context) -> i32 {
    if args.len() > 1 {
        println!("Invalid arguments! Expected less than 2, got {}", args.len());

        return 1;
    }

    if args.is_empty() {
        let keys = ctx.variables.keys();
        println!("{:?}", keys);

        return 0;
    }

    let mut var_name = args[0].clone();
    let is_env = if var_name.starts_with("ENV:") {
        let (_, var_name_) = var_name.split_at(4);
        var_name = var_name_.to_string();

        true
    } else {
        false
    };

    // Ensure alphanumeric or '_'
    if let Some(pos) = var_name
        .chars()
        .position(|ch| !((ch.is_alphanumeric() || ch == '_') && ch != ' '))
    {
        let invalid_char = var_name.chars().nth(pos).unwrap_or_default();
        println!("Invalid character at position {}, '{}'", pos, invalid_char);

        return 1;
    }

    let val = if is_env {
        env::var(var_name).unwrap_or_default()
    } else {
        ctx.variables.get(&var_name).unwrap_or(&String::new()).clone()
    };
    println!("{}", val);

    0
}
