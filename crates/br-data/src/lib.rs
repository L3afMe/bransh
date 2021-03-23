pub mod options;
pub mod context;
pub mod command;

use std::env;

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
