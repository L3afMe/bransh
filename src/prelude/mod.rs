use std::io::{Stdout, stdout};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::options::Options;
pub use crate::util::tabcomp::*;

pub struct Context<'t> {
    pub command_buffer: String,
    pub prompt: String,
    pub cursor_pos: (u16, u16),
    pub current_key: KeyEvent,
    pub last_key: KeyEvent,
    pub tab: TabContext,
    pub options: &'t Options,
    pub writer: Stdout,
}

impl<'t> Context<'t> {
    pub fn new(options: &'t Options) -> Self {
        Self {
            command_buffer: String::new(),
            prompt: String::new(),
            cursor_pos: (0, 0),
            current_key: KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            last_key: KeyEvent::new(KeyCode::Null, KeyModifiers::NONE),
            tab: TabContext::new(),
            options,
            writer: stdout()
        }
    }
}
