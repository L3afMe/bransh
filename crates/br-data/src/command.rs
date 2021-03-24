use std::fmt;

use crate::context::Context;

pub type ExecuteFn = fn(args: Vec<String>, ctx: &mut Context) -> i32;
pub type TabCompletionFn = fn(Vec<String>, &Context) -> Vec<String>;

#[derive(Clone)]
pub enum TabCompletionType {
    None,
    Directory(Vec<TabCompletion>),
    File(Vec<TabCompletion>),
    FileOrDirectory(Vec<TabCompletion>),
    Static(Vec<TabCompletion>),
    Dynamic(TabCompletionFn),
}

#[derive(Clone)]
pub struct TabCompletion {
    pub arg: String,
    pub subargs: TabCompletionType,
}

impl TabCompletion {
    pub fn new(arg: &str, subargs: TabCompletionType) -> Self {
        Self {
            arg: arg.to_string(),
            subargs
        }
    }
}

#[derive(Clone)]
pub struct BrBuiltin {
    pub name: &'static str,
    pub execute: ExecuteFn,
    pub tab_completion: TabCompletionType,
}

impl fmt::Debug for BrBuiltin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrBuiltin").field("name", &self.name).finish()      
    }
}
