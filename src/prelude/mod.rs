use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    str::FromStr,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::cli::{history::HistoryContext, tabcomp::TabContext, util::print_cmd_buf};

#[derive(Debug, Default, Clone)]
pub struct CommandBufferBackup {
    pub buffer: String,
    pub cursor: u16,
}

impl CommandBufferBackup {
    pub fn new(buffer: String, cursor: u16) -> Self {
        Self {
            buffer,
            cursor,
        }
    }
}

pub struct Context {
    pub command_buffer: String,
    pub prompt:         String,
    pub cursor_pos:     (u16, u16),
    pub current_key:    KeyEvent,
    pub last_key:       KeyEvent,
    pub tab:            TabContext,
    pub history:        HistoryContext,
    pub writer:         Stdout,
    pub variables:      HashMap<String, String>,
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
            command_buffer: String::new(),
            prompt:         String::new(),
            cursor_pos:     (0, 0),
            current_key:    KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            last_key:       KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            tab:            TabContext::default(),
            history:        HistoryContext::default(),
            writer:         stdout(),
            variables:      vars,
        }
    }
}

impl Context {
    pub fn prompt_len(&self) -> usize {
        self.prompt.chars().count()
    }

    pub fn restore_backup(&mut self, backup: &CommandBufferBackup) {
        self.command_buffer = backup.buffer.clone();
        let new_pos = (backup.cursor as i16) - (self.cursor_pos.0 as i16);
        print_cmd_buf(self, new_pos);
    }

    pub fn get_variable<T: FromStr>(&self, var_name: &str, default: T) -> T {
        let str_val_wrapped = self.variables.get(var_name);

        if let Some(str_var) = str_val_wrapped {
            if let Ok(val) = str_var.parse::<T>() {
                return val;
            }
        }

        default
    }
}
