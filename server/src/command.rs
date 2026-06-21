use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct QueuedCommand {
    pub name: String,
}

impl QueuedCommand {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Default)]
pub struct CommandQueue {
    commands: VecDeque<QueuedCommand>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn push(&mut self, command: QueuedCommand, maximum_size: usize) -> bool {
        if self.commands.len() >= maximum_size {
            return false;
        }

        self.commands.push_back(command);
        true
    }

    pub fn pop(&mut self) -> Option<QueuedCommand> {
        self.commands.pop_front()
    }

    pub fn front(&self) -> Option<&QueuedCommand> {
        self.commands.front()
    }
}
