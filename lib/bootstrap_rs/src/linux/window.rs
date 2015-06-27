use std::rc::Rc;
use std::cell::RefCell;

use window::Message;

#[derive(Debug, Clone, Copy)]
pub struct Window;

impl Window {
    pub fn new(_name: &str, _instance: ()) -> Rc<RefCell<Window>> {
        println!("Window::new() is not implemented on linux");
        Rc::new(RefCell::new(Window))
    }

    pub fn handle_messages(&mut self) {
        println!("Window::handle_messages() is not implemented on linux");
    }

    pub fn next_message(&mut self) -> Option<Message> {
        println!("Window::next_message() is not implemented on linux");
        None
    }
}
