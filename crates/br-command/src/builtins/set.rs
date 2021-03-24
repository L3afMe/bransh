use br_data::{
    command::{BrBuiltin, ExecuteFn, TabCompletionType},
    context::Context,
};

use super::get::tc_var_list;

pub const CMD: BrBuiltin = BrBuiltin {
    name: "set",
    tab_completion: TabCompletionType::Dynamic(tc_var_list),
    execute,
};

#[allow(non_upper_case_globals)]
const execute: ExecuteFn = |args: Vec<String>, ctx: &mut Context| -> i32 {
    if args.len() != 2 {
        println!("Invalid arguments! Expected 2, got {}", args.len());

        return 1;
    }

    let mut var_name = args[0].clone();
    let var_value = args[1].clone();
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

    ctx.set_variable(&var_name, var_value, is_env);

    0
};
