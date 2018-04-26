use std::{collections::VecDeque, sync::Mutex};
use val::Value;

#[derive(Debug)]
pub struct Channel {
    messages: Mutex<VecDeque<Value>>,
}

impl Channel {
    pub fn recv(&self) -> Option<Value> {
        self.messages.lock().unwrap().pop_front()
    }

    pub fn send(&self, val: Value) {
        self.messages.lock().unwrap().push_back(val);
    }

    pub fn new() -> Self {
        Self {
            messages: Mutex::new(VecDeque::with_capacity(8)),
        }
    }
}
