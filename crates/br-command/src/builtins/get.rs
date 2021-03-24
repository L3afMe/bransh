use br_data::{
    command::{BrBuiltin, ExecuteFn, TabCompletionFn, TabCompletionType},
    context::Context,
};

pub const CMD: BrBuiltin = BrBuiltin {
    name: "get",
    tab_completion: TabCompletionType::Dynamic(tc_var_list),
    execute,
};

#[allow(non_upper_case_globals)]
const tc_var_list: TabCompletionFn =
    |_args: Vec<String>, ctx: &Context| -> Vec<String> { ctx.aliases.keys().map(|key| key.to_string()).collect() };

#[allow(non_upper_case_globals)]
const execute: ExecuteFn = |args: Vec<String>, ctx: &mut Context| -> i32 {
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

    let val = ctx.get_variable(&var_name, String::new(), is_env);
    println!("{}", val);

    0
};
