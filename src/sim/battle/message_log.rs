#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageLog {
    messages: Vec<String>,
    last_turn_cursor: usize,
}

const INITIAL_MESSAGE_LOG_CAPACITY: usize = 200;
impl MessageLog {
    pub fn new() -> Self {
        Self {
            messages: Vec::with_capacity(INITIAL_MESSAGE_LOG_CAPACITY),
            last_turn_cursor: 0,
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

    pub fn show_last_turn_messages(&self) {
        self.messages[self.last_turn_cursor..].iter().for_each(|message| {
            println!("{}", message);
        })
    }
    
    pub fn snap_last_turn_cursor_to_end(&mut self) {
        self.last_turn_cursor = self.len()
    }

    pub fn push(&mut self, message: impl ToString) {
        self.messages.push(message.to_string());
    }

    pub fn extend(&mut self, messages: &[&str]) {
        for message in messages {
            self.messages.push(message.to_string());
        }
    }
    
    pub(crate) fn show_last_message(&self) {
        println!("{}", self.messages[self.len()-1]);
    }
}