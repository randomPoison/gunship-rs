use std::rc::Rc;
use std::cell::RefCell;

use window::Message;

pub struct Window;

impl Window {
    pub fn new(name: &str, _instance: ()) -> Rc<RefCell<Window>> {
        Rc::new(RefCell::new(Window))
    }

    pub fn handle_messages(&mut self) {

    }

    pub fn next_message(&mut self) -> Option<Message> {
        None
    }
}
