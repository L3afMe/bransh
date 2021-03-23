mod commands;

use br_data::context::Context;

pub fn load_builtins(ctx: &mut Context) {
    ctx.builtins = vec![
        commands::cd::CMD,
        commands::get::CMD,
        commands::set::CMD,
        commands::alias::CMD,
    ];
}
