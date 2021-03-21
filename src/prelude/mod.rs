use std::io::{stdout, Stdout};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{cli::{history::HistoryContext, tabcomp::TabContext, util::print_cmd_buf}, options::Options};

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
    pub options:        Options,
    pub writer:         Stdout,
}

impl Context {
    pub fn new(options: Options) -> Self {
        Self {
            command_buffer: String::new(),
            prompt: String::new(),
            cursor_pos: (0, 0),
            current_key: KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            last_key: KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            tab: TabContext::default(),
            history: HistoryContext::default(),
            options,
            writer: stdout(),
        }
    }

    pub fn prompt_len(&self) -> usize {
        self.prompt.chars().count()
    }

    pub fn restore_backup(&mut self, backup: &CommandBufferBackup) {
        self.command_buffer = backup.buffer.clone();
        let new_pos = (backup.cursor as i16) - (self.cursor_pos.0 as i16);
        print_cmd_buf(self, new_pos);
    }
}
