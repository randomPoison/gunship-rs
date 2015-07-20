use std::rc::Rc;
use std::cell::RefCell;

#[macro_use]
extern crate objc;
extern crate cocoa;
extern crate core_graphics;
extern crate core_foundation;

use cocoa::foundation::*;
use cocoa::appkit::*;
use cocoa::base::*;

use window::Message;

pub struct Window {
    window: NSWindow
}

impl Window {
    pub fn new(name: &str, _instance: ()) -> Rc<RefCell<Window>> { unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let point = NSPoint { x: 0.0, y: 0.0 };
        let size = NSSize { width: 500.0, height: 500.0 };
        let frame = NSRect { origin: point, size: size };

        let window = NSWindow::alloc(nil);
        window.initWithContentRect_styleMask_backing_defer_(frame,
                                                            NSWindowMask::NSTitledWindowMask as NSUInteger|
                                                            NSWindowMask::NSClosableWindowMask as NSUInteger|
                                                            NSWindowMask::NSMiniaturizableWindowMask as NSUInteger|
                                                            NSWindowMask::NSResizableWindowMask as NSUInteger,
                                                            NSBackingStoreType::NSBackingStoreBuffered,
                                                            NO).autorelease();

        window.cascadeTopLeftFromPoint_(NSPoint::new(20.0, 20.0));
        window.center();
        let title = NSString::alloc(nil).init_str("Poop");
        window.setTitle_(title);
        window.setAcceptsMouseMovedEvents_(YES);

        let frame = window.contentRectForFrameRect_(frame);
        //let content = NSView::alloc(nil);
        let content = NSView::initWithFrame_ (nil, frame);
        //content = NSView::initWithFrame_(content, frame);
        window.setContentView_(content);

        window.makeKeyAndOrderFront_(nil);
        window.orderFrontRegardless();

        Rc::new(RefCell::new(Window {
            window: window,
        }))
    } }

    pub fn next_message(&mut self) -> Option<Message> {
        None
    }
}
