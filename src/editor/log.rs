use std::collections::HashSet;

use crate::core::Mesh;

use super::{object::Object, tool::Tool};

/// An action is a message in the application log, the ActionTree.
/// Actions are re- and undoable.
pub enum Message {
    Tool(Tool),
    Selection(HashSet<u32>),
    Object { id: u32, object: Option<Object> },
    Mesh { id: u32, mesh: Mesh },
    None,
}

impl Default for Message {
    fn default() -> Self {
        Self::None
    }
}

pub const UNDO_CAPACITY: usize = 20;

/// The stack allocated application log. It stores the last n actions that occured for undoing
/// and redoing, where n is the number stored in the UNDO_CAPACITY constant.
pub struct Log {
    pub messages: Vec<Message>,
    pub cursor: usize,
    before: usize,
    after: usize,
}

impl Log {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        let mut actions = Vec::with_capacity(capacity);
        for _ in 0..actions.capacity() {
            actions.push(Message::None);
        }

        Self {
            messages: actions,
            cursor: 0,
            before: 0,
            after: 0,
        }
    }

    /// Pushes an action. This returns the action that was overwritten (if any).
    pub fn push(&mut self, mut message: Message) -> Option<Message> {
        if matches!(message, Message::None) {
            return None;
        }

        self.cursor = (self.cursor + 1) % self.messages.len();
        self.after = 0;

        if self.before == self.messages.len() {
            std::mem::swap(&mut message, &mut self.messages[self.cursor]);
            Some(message)
        } else {
            self.before += 1;
            self.messages[self.cursor] = message;
            None
        }
    }

    pub fn cursor_mut(&mut self) -> &mut Message {
        &mut self.messages[self.cursor]
    }

    pub fn can_undo(&self) -> bool {
        self.before > 0
    }

    /// Retracts the cursor
    pub fn undo(&mut self) {
        self.cursor = self
            .cursor
            .checked_sub(1)
            .unwrap_or(self.messages.len() - 1);
        self.after = (self.after + 1).min(self.messages.len());
        self.before -= 1;
    }

    pub fn can_redo(&self) -> bool {
        self.after > 0
    }

    /// Advances the cursor
    pub fn redo(&mut self) {
        self.cursor = (self.cursor + 1) % self.messages.len();
        self.before = (self.before + 1).min(self.messages.len());
        self.after -= 1;
    }
}

impl Default for Log {
    fn default() -> Self {
        Self::new(UNDO_CAPACITY)
    }
}
