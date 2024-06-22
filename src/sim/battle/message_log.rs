#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLog {
    messages: Vec<String>,
    shown_messages_cursor: usize,
}

impl Default for MessageLog {
    fn default() -> Self {
        Self::new()
    }
}

const INITIAL_MESSAGE_LOG_CAPACITY: usize = 200;
impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::with_capacity(INITIAL_MESSAGE_LOG_CAPACITY),
            shown_messages_cursor: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn as_vec(&self) -> &Vec<String> {
        &self.messages
    }

    pub fn show_all_messages(&self) {
        self.messages.iter().for_each(|message| {
            println!("{}", message);
        })
    }

    pub fn show_new_messages(&mut self) {
        for message in &self.messages[self.shown_messages_cursor..] {
            println!("{}", message);
        }
        self.shown_messages_cursor = self.len()
    }

    pub fn push(&mut self, message: impl ToString) {
        self.messages.push(message.to_string());
    }

    pub fn extend(&mut self, messages: &[impl ToString]) {
        for message in messages {
            self.messages.push(message.to_string());
        }
    }
}
