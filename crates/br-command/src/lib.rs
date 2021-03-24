#[macro_use]
extern crate lazy_static;

mod builtins;

use br_data::{command::TabCompletionType, context::Context};

pub fn load_builtins(ctx: &mut Context) {
    ctx.builtins = vec![
        builtins::alias::CMD.clone(),
        builtins::cd::CMD,
        builtins::exit::CMD,
        builtins::get::CMD,
        builtins::set::CMD,
    ];
}

pub fn get_tab_completion(cmd: String, args: Vec<String>, ctx: &mut Context) -> Vec<String> {
    for builtin in ctx.builtins.clone() {
        if builtin.name == cmd {
            return get_comp(builtin.tab_completion, args, ctx);
        }
    }

    Vec::new()
}

fn get_comp(tc_type: TabCompletionType, args: Vec<String>, ctx: &mut Context) -> Vec<String> {
    match tc_type {
        TabCompletionType::File
        | TabCompletionType::Directory
        | TabCompletionType::FileOrDirectory
        | TabCompletionType::None => Vec::new(),
        TabCompletionType::Dynamic(fun) => (fun)(args, ctx),
        TabCompletionType::Static(comp) => {
            if args.is_empty() {
                return comp.into_iter().map(|arg| arg.arg).collect();
            }

            let mut comp = Some(comp);

            let mut itr = args.into_iter().peekable();

            'comploop: while comp.is_some() {
                let arg = itr.next().unwrap();

                // If is last arg
                if itr.peek().is_none() {
                    return comp
                        .unwrap()
                        .into_iter()
                        .map(|comp| comp.arg)
                        .filter(|comp| comp.starts_with(&arg))
                        .collect();
                }

                for comp_arg in comp.unwrap() {
                    if comp_arg.arg == arg {
                        match comp_arg.subargs {
                            TabCompletionType::Static(val) => comp = Some(val),
                            tc => return get_comp(tc, itr.collect(), ctx),
                        }

                        continue 'comploop;
                    }
                }

                comp = None
            }
            Vec::new()
        },
    }
}
