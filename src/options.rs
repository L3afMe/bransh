use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct PromptOptions {
    pub format: String,
    pub truncate_home: bool,
    pub truncate_home_symbol: String,
    pub truncate_directories: u8,
    pub truncate_directories_symbol: String,
}

impl Default for PromptOptions {
    fn default() -> Self {
        Self {
            format: String::from("{WD} | "),
            truncate_home: true,
            truncate_home_symbol: String::from("~"),
            truncate_directories: 2,
            truncate_directories_symbol: String::from("…"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub syntax_highlighting: bool,
    pub paths: Vec<PathBuf>,
    pub prompt: PromptOptions,
}

impl Default for Options {
    fn default() -> Self {
        let paths = match env::var_os("PATH") {
            Some(paths) => env::split_paths(&paths).collect(),
            None => Vec::new(),
        };
        Self {
            syntax_highlighting: true,
            paths,
            prompt: PromptOptions::default(),
        }
    }
}

pub fn load_options() -> Options {
    Options::default()
}

pub fn get_config_dir() -> Option<String> {
    if cfg!(unix) {
        let config_var = env::var("XDG_CONFIG_DIR");
        if let Ok(config) = config_var {
            return Some(format!("{}/bransh/", config));
        }

        let home_path = home::home_dir()?;
        let home = home_path.to_str()?;
        Some(format!("{}/.config/bransh", home))
    } else {
        // TODO: add Windows support
        None
    }
}
