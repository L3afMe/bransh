use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Options {
    pub syntax_highlighting: bool,
    pub paths: Vec<PathBuf>
}

impl Default for Options {
    fn default() -> Self {
        let paths = match env::var_os("PATH") {
            Some(paths) => env::split_paths(&paths).collect(),
            None => Vec::new()
        };
        Self {
            syntax_highlighting: true,
            paths,
        }
    }
}

pub fn load_options() -> Options {
    Options::default()
}
