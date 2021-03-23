use std::fmt;

use crate::context::Context;

pub type ExecuteFn = fn(args: Vec<String>, ctx: &mut Context) -> i32;

#[derive(Clone)]
pub struct BrBuiltin {
    pub name: &'static str,
    pub execute: ExecuteFn,
}

impl fmt::Debug for BrBuiltin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrBuiltin").field("name", &self.name).finish()      
    }
}
