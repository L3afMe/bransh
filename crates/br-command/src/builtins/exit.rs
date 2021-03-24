use br_data::{
    command::{BrBuiltin, ExecuteFn, TabCompletionType},
    context::Context,
};

pub const CMD: BrBuiltin = BrBuiltin {
    name: "exit",
    tab_completion: TabCompletionType::None,
    execute,
};

#[allow(non_upper_case_globals)]
const execute: ExecuteFn = |_args: Vec<String>, _ctx: &mut Context| -> i32 {
    // Dummy command as exit is hardcoded into br-executer,
    // this is soley for tab completion and validating commands
    0
};
