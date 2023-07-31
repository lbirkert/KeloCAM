use std::collections::{HashMap, HashSet};

use super::{
    log::{Log, Message},
    object::Object,
    tool::Tool,
};

#[derive(Default)]
pub struct State {
    pub selection: HashSet<u32>,
    pub objects: HashMap<u32, Object>,
    // Used for displaying the objects in the correct order
    pub object_ids: Vec<u32>,
    pub tool: Tool,

    id_counter: u32,
    log: Log,
}

impl State {
    // --- MESSAGES ---
    // Utility methods for constructing messages to modify this state.

    pub fn perform_selection(&self, ui: &egui::Ui, id: u32) -> Message {
        if ui.input(|i| i.modifiers.shift) {
            let mut selection = self.selection.clone();
            if selection.contains(&id) {
                selection.remove(&id);
            } else {
                selection.insert(id);
            }
            Message::Selection(selection)
        } else if self.selection.len() == 1 && self.selection.iter().next().unwrap() == &id {
            Message::None
        } else {
            let mut selection = HashSet::new();
            selection.insert(id);
            Message::Selection(selection)
        }
    }

    pub fn unselect_all(&self) -> Message {
        Message::Selection(HashSet::new())
    }

    pub fn select_all(&self) -> Message {
        Message::Selection(HashSet::from_iter(self.objects.clone().into_keys()))
    }

    pub fn insert_object(&self, object: Object) -> Message {
        Message::Object {
            object: Some(object),
            id: 0,
        }
    }

    pub fn delete_object(&self, id: u32) -> Message {
        Message::Object { object: None, id }
    }

    // --- MESSAGES ---

    /// Returns whether any object is currently selected.
    pub fn selected(&self) -> bool {
        !self.selection.is_empty()
    }

    /// Iterate over the objects contained in the selection.
    pub fn iter_selection(&self) -> impl Iterator<Item = (&u32, &Object)> {
        self.selection
            .iter()
            .filter(|id| self.objects.contains_key(id))
            .map(|id| (id, &self.objects[id]))
    }

    /// Undo
    pub fn undo(&mut self) {
        if self.log.can_undo() {
            self.apply();
            self.log.undo();
        }
    }

    /// Redo
    pub fn redo(&mut self) {
        if self.log.can_redo() {
            self.log.redo();
            self.apply();
        }
    }

    /// Pushes a message onto the state. This includes applying the message & revoking objects out of the
    /// object table if the overwritten message is a delete message (aka. the deletion cannot be undone)
    pub fn push(&mut self, message: Message) {
        self.log.push(message);
        self.apply();
    }

    /// Applies (aka. executes) the message under the cursor on this state. Most messages are
    /// double-cycled, which means that when the return value of this function gets applied,
    /// the state before the first application persists.
    pub fn apply(&mut self) {
        match &mut self.log.actions[self.log.cursor] {
            Message::Object {
                ref mut id,
                ref mut object,
            } => {
                let mut tmp = None;
                std::mem::swap(&mut tmp, object);
                if let Some(object) = tmp {
                    if *id == 0 {
                        self.id_counter += 1;
                        *id = self.id_counter;
                    }

                    self.object_ids.push(*id);
                    self.objects.insert(*id, object);
                } else {
                    self.object_ids
                        .remove(self.object_ids.iter().position(|i| i == id).unwrap());
                    *object = self.objects.remove(id);
                }
            }
            Message::Mesh { id, ref mut mesh } => {
                self.tool.reset();
                let object = self.objects.get_mut(id).unwrap();
                std::mem::swap(mesh, &mut object.mesh);
            }
            Message::Tool(ref mut tool) => {
                std::mem::swap(tool, &mut self.tool);
            }
            Message::Selection(ref mut selection) => {
                std::mem::swap(selection, &mut self.selection);
            }
            _ => {}
        }
    }
}
