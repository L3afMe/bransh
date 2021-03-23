use std::{
    collections::HashMap,
    env,
    io::{stdout, Stdout},
    str::FromStr,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::command::BrBuiltin;

#[derive(Debug, Default, Clone)]
pub struct CommandBufferBackup {
    pub buffer: String,
    pub cursor: u16,
}

impl CommandBufferBackup {
    pub const fn new(buffer: String, cursor: u16) -> Self {
        Self {
            buffer,
            cursor,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CompletionContext {
    pub list:   Vec<String>,
    pub index:  u16,
    pub backup: CommandBufferBackup,
}

#[derive(Debug, Clone)]
pub struct CliContext {
    pub command_buffer: String,
    pub prompt:         String,
    pub cursor_pos:     (u16, u16),
    pub current_key:    KeyEvent,
    pub last_key:       KeyEvent,
    pub completion:     CompletionContext,
}

impl Default for CliContext {
    fn default() -> Self {
        Self {
            command_buffer: String::new(),
            prompt:         String::new(),
            cursor_pos:     (0, 0),
            current_key:    KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            last_key:       KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            completion:     CompletionContext::default(),
        }
    }
}

impl CliContext {
    pub fn prompt_len(&self) -> usize {
        self.prompt.chars().count()
    }
}

pub struct Context {
    pub cli:       CliContext,
    pub writer:    Stdout,
    pub variables: HashMap<String, String>,
    pub aliases:   HashMap<String, String>,
    pub builtins:  Vec<BrBuiltin>,
}

impl Default for Context {
    fn default() -> Self {
        let mut vars = HashMap::new();

        vars.insert(String::from("PROMPT"), String::from("{WD} | "));
        vars.insert(String::from("P_HOME_TRUNC"), String::from("true"));
        vars.insert(String::from("P_HOME_CHAR"), String::from("~"));
        vars.insert(String::from("P_DIR_TRUNC"), String::from("2"));
        vars.insert(String::from("P_DIR_CHAR"), String::from("…"));
        vars.insert(String::from("SYN_HIGHLIGHTING"), String::from("true"));

        Self {
            cli:       CliContext::default(),
            writer:    stdout(),
            variables: vars,
            aliases:   HashMap::new(),
            builtins:  Vec::new(),
        }
    }
}

impl Context {
    pub fn get_variable<T: FromStr>(&self, var_name: &str, default: T, env: bool) -> T {
        let str_val = if env {
            if let Ok(val) = env::var(var_name) {
                val
            } else {
                return default;
            }
        } else if let Some(val) = self.variables.get(var_name) {
            val.clone()
        } else {
            return default;
        };

        if let Ok(val) = str_val.parse::<T>() {
            return val;
        }

        default
    }

    pub fn set_variable<T: ToString>(&mut self, var_name: &str, var_value: T, is_env: bool) {
        if is_env {
            env::set_var(var_name, var_value.to_string());
        } else {
            self.variables.insert(var_name.to_string(), var_value.to_string());
        }
    }
}
