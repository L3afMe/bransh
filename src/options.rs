use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct PromptOptions {
    pub format: String,
    pub truncate_home: bool,
    pub truncate_directories: u8,
}

impl Default for PromptOptions {
    fn default() -> Self {
        Self {
            format: String::from("{USER}@{HOST}:{PWD}$ "),
            truncate_home: true,
            truncate_directories: 0,
        }
    }

}

#[derive(Debug, Clone)]
pub struct Options {
    pub syntax_highlighting: bool,
    pub paths: Vec<PathBuf>,
    pub prompt: PromptOptions
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
            prompt: PromptOptions::default()
        }
    }
}

pub fn load_options() -> Options {
    Options::default()
}
