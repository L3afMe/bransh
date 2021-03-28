use br_data::{
    command::{BrBuiltin, ExecuteFn, TabCompletion, TabCompletionFn, TabCompletionType},
    context::Context,
};

lazy_static! {
    pub static ref CMD: BrBuiltin = BrBuiltin {
        name: "alias",
        tab_completion: TabCompletionType::Static(vec![
            TabCompletion::new("list", TabCompletionType::None),
            TabCompletion::new("del", TabCompletionType::Dynamic(tc_alias_list)),
            TabCompletion::new("get", TabCompletionType::Dynamic(tc_alias_list)),
            TabCompletion::new("set", TabCompletionType::None)
        ]),
        execute,
    };
}

#[allow(non_upper_case_globals)]
const tc_alias_list: TabCompletionFn = |args: Vec<String>, ctx: &Context| -> Vec<String> {
    if args.len() <= 1 {
        let cur_arg = if let Some(arg) = args.get(0) {
            arg.clone()
        } else {
            String::new()
        };
        ctx.aliases
            .keys()
            .map(|key| key.to_string())
            .filter(|key| key.starts_with(&cur_arg))
            .collect()
    } else {
        Vec::new()
    }
};

#[allow(non_upper_case_globals)]
const execute: ExecuteFn = |mut args: Vec<String>, ctx: &mut Context| -> i32 {
    if args.is_empty() {
        eprintln!("Invalid arguments! Expected 1-2, got 0");

        return 1;
    }

    let operator = args[0].clone();
    args.remove(0);

    match operator.as_ref() {
        "get" => get_alias(args, ctx),
        "set" => set_alias(args, ctx),
        "del" => del_alias(args, ctx),
        "list" => list_aliases(ctx),
        _ => {
            eprintln!("Invalid argument at pos 1! Expected one of 'get', 'set', 'del' or 'list'");

            1
        },
    }
};

fn get_alias(args: Vec<String>, ctx: &mut Context) -> i32 {
    if args.is_empty() {
        eprintln!("Invalid arguments! Expected 2, got 1");

        return 1;
    }

    let key = args[0].clone();
    if ctx.aliases.contains_key(&key) {
        println!(
            "{}",
            ctx.aliases
                .get(&key)
                .unwrap_or(&String::from("Error occured while getting alias"))
        );

        return 0;
    }

    eprintln!("Unable to find alias '{}'!", key);

    1
}

fn set_alias(args: Vec<String>, ctx: &mut Context) -> i32 {
    if args.len() != 2 {
        eprintln!("Invalid arguments! Expected 3, got {}", args.len() + 1);

        return 1;
    }

    let key = args[0].clone();
    let value = args[1].clone();
    ctx.aliases.insert(key, value);

    0
}

fn del_alias(args: Vec<String>, ctx: &mut Context) -> i32 {
    if args.is_empty() {
        eprintln!("Invalid arguments! Expected 2, got 1");

        return 1;
    }

    let key = args[0].clone();
    if ctx.aliases.contains_key(&key) {
        ctx.aliases.remove(&key);

        return 0;
    }

    eprintln!("Unable to find alias '{}'!", key);

    1
}

fn list_aliases(ctx: &mut Context) -> i32 {
    println!("{:?}", ctx.aliases.keys());

    0
}
