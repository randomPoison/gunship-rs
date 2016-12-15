#![allow(non_snake_case)]

use platform::cocoa::appkit::*;
use platform::cocoa::foundation::*;
use platform::cocoa::base::{nil};
use objc::declare::*;
use objc::runtime::*;
use std::collections::VecDeque;
use std::os::raw::c_void;
use window::Message;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct ObjcObject(*mut Object);

unsafe impl Send for ObjcObject {}

mod window_map {
    use cell_extras::AtomicInitCell;
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Mutex, Once, ONCE_INIT};
    use super::ObjcObject;
    use window::Message;

    static WINDOW_MAP: AtomicInitCell<Mutex<WindowMap>> = AtomicInitCell::new();
    static WINDOW_MAP_INIT: Once = ONCE_INIT;

    pub type WindowMap = HashMap<ObjcObject, VecDeque<Message>>;

    pub fn with<F, T>(func: F) -> T
        where F: FnOnce(&mut WindowMap) -> T
    {
        // Initialize the map once.
        WINDOW_MAP_INIT.call_once(|| {
            WINDOW_MAP.init(Mutex::new(HashMap::new()));
        });

        // Lock the mutex and invoke `func` with the `WindowMap`.
        let mutex = WINDOW_MAP.borrow();
        let mut guard = mutex.lock().expect("Window mutex was poisoned ☠");
        func(&mut *guard)
    }
}

#[derive(Debug)]
pub struct Window {
    app: *mut Object,
    _window: *mut Object,
}

impl Window {
    pub fn new(_name: &str) -> Window {
        // Grab Objective C types.
        let NSApplication = Class::get("NSApplication").unwrap();
        let NSAutoreleasePool = Class::get("NSAutoreleasePool").unwrap();

        let pool: *mut Object = unsafe { msg_send![NSAutoreleasePool, new] };

        // Create custom NSApplication subclass.
        let MyApplication = {
            let mut class_decl = ClassDecl::new("MyApplication", NSApplication).unwrap();
            unsafe {
                class_decl.add_method(
                    sel!(run),
                    run as extern fn (&mut Object, Sel)
                );
                class_decl.add_method(
                    sel!(windowWillClose:),
                    window_will_close as extern fn (&mut Object, Sel, *mut Object),
                );
                class_decl.add_ivar::<*mut c_void>("bootstrapWindow");
                //class_decl.add_method(
                //    sel!(sendEvent:),
                //    send_event as extern fn (&mut Object, Sel, *mut Object),
                //);
            }
            class_decl.register()
        };

        // Initialize and run the application.
        // ===================================

        // Create and retrieve the application instance.
        let app: *mut Object = unsafe { msg_send![MyApplication, sharedApplication] };

        // Assign the app delegate to the application instance and run the appliction.
        let window = unsafe {
            //msg_send![app, run];
            msg_send![app,
                performSelectorOnMainThread: sel!(run)
                withObject: nil
                waitUntilDone: YES];

            let window = open_window(app);

            msg_send![pool, release];

            Window {
                app: app,
                _window: window,
            }
        };

        // Initialize the window map if necessary and add the window to it.
        window_map::with(|window_map| {
            window_map.insert(ObjcObject(window.app), VecDeque::default())
        });

        window
    }

    pub fn close(&self) {
        unimplemented!();
        // unsafe { self.window.performClose_(nil); }
    }

    // TODO: Implement non-blocking window messages.
    pub fn next_message(&mut self) -> Option<Message> {
        None
    }

    /// Waits for the next window message, blocking if none is pending.
    pub fn wait_message(&mut self) -> Option<Message> {
        loop {
            // First check if there are any pending messages in the window map.
            let pending_message = window_map::with(|window_map| {
                let messages = window_map.get_mut(&ObjcObject(self.app)).expect("Unable to find window in window map");
                messages.pop_front()
            });

            if let Some(message) = pending_message {
                return Some(message);
            }

            unsafe {
                // TODO: Create autorelease blocks?

                let NSDate = Class::get("NSDate").unwrap();
                let distant_future = msg_send![NSDate, distantFuture];

                //let NSApplication = Class::get("NSApplication").unwrap();
                //let event: *mut Object = msg_send![
                //    super(self.app, NSApplication),
                //    nextEventMatchingMask: 0xffffffff //NSAnyEventMask,
                //    untilDate: distant_future
                //    inMode: NSDefaultRunLoopMode
                //    dequeue: YES
                //];

                // HACK: For some reason the above doesn't work. I don't know why, but we should fix it.
                let imp = get_next_message_imp();

                let event = imp(
                    self.app,
                    sel!(nextEventMatchingMask:untilDate:inMode:dequeue:),
                    0xffffffff, //NSAnyEventMask,
                    distant_future,
                    NSDefaultRunLoopMode,
                    YES,
                );

                msg_send![self.app, sendEvent:event];
                msg_send![self.app, updateWindows];

                if let Some(event) = map_event(event) {
                    return Some(event);
                }
            }
        }
    }

    pub fn get_rect(&self) -> (i32, i32, i32, i32) {
        (0, 0, 1, 1)
    }

    pub fn inner(&self) -> WindowInner {
        WindowInner(self.app)
    }
}

impl<'a> IntoIterator for &'a mut Window {
    type Item = Message;
    type IntoIter = WindowMessages<'a>;

    fn into_iter(self) -> WindowMessages<'a> {
        WindowMessages(self)
    }
}

pub struct WindowInner(*mut Object);

impl WindowInner {
    pub fn wait_message(&mut self) -> Option<Message> {
        loop {
            // First check if there are any pending messages in the window map.
            let pending_message = window_map::with(|window_map| {
                let messages = window_map.get_mut(&ObjcObject(self.0)).expect("Unable to find window in window map");
                messages.pop_front()
            });

            if let Some(message) = pending_message {
                return Some(message);
            }

            unsafe {
                // TODO: Create autorelease blocks?

                let NSDate = Class::get("NSDate").unwrap();
                let distant_future = msg_send![NSDate, distantFuture];

                //let NSApplication = Class::get("NSApplication").unwrap();
                //let event: *mut Object = msg_send![
                //    super(self.0, NSApplication),
                //    nextEventMatchingMask: 0xffffffff //NSAnyEventMask,
                //    untilDate: distant_future
                //    inMode: NSDefaultRunLoopMode
                //    dequeue: YES
                //];

                // HACK: For some reason the above doesn't work. I don't know why, but we should fix it.
                let imp = get_next_message_imp();

                let event = imp(
                    self.0,
                    sel!(nextEventMatchingMask:untilDate:inMode:dequeue:),
                    0xffffffff, //NSAnyEventMask,
                    distant_future,
                    NSDefaultRunLoopMode,
                    YES,
                );

                msg_send![self.0, sendEvent:event];
                msg_send![self.0, updateWindows];

                if let Some(event) = map_event(event) {
                    return Some(event);
                }
            }
        }
    }

    pub fn pump_forever(&mut self) {

    }
}

pub struct WindowMessages<'a>(&'a mut Window);

impl<'a> Iterator for WindowMessages<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.0.wait_message()
    }
}

fn map_event(event: *mut Object) -> Option<Message> {
    use platform::cocoa::appkit::NSEventType::*;
    use window::Message::*;
    use input::ScanCode;

    let message = match unsafe { msg_send![event, type] } {
        NSLeftMouseDown => Message::MouseButtonPressed(0),
        NSLeftMouseUp => Message::MouseButtonReleased(0),
        NSRightMouseDown => Message::MouseButtonPressed(1),
        NSRightMouseUp => Message::MouseButtonReleased(1),
        NSMouseMoved | NSLeftMouseDragged | NSRightMouseDragged => {
            let pos: NSPoint = unsafe { msg_send![event, locationInWindow] };
            MousePos(pos.x as i32, pos.y as i32)
        },
        //NSMouseEntered => !,
        //NSMouseExited => !,
        NSKeyDown => KeyDown(ScanCode::Unsupported),
        NSKeyUp => KeyUp(ScanCode::Unsupported),
        //NSFlagsChanged => !,
        //NSAppKitDefined => !,
        //NSSystemDefined => !,
        //NSApplicationDefined => !,
        //NSPeriodic => !,
        //NSCursorUpdate => !,
        NSScrollWheel => MouseWheel(0),
        //NSTabletPoint => !,
        //NSTabletProximity => !,
        //NSOtherMouseDown => !,
        //NSOtherMouseUp => !,
        //NSOtherMouseDragged => !,
        //NSEventTypeGesture => !,
        //NSEventTypeMagnify => !,
        //NSEventTypeSwipe => !,
        //NSEventTypeRotate => !,
        //NSEventTypeBeginGesture => !,
        //NSEventTypeEndGesture => !,
        //NSEventTypeSmartMagnify => !,
        //NSEventTypeQuickLook => !,
        //NSEventTypePressure => !,

        _ => return None,
    };

    Some(message)
}

extern fn run(
    app: &mut Object,
    _sel: Sel
) {
    // TODO: Create autorelease blocks?

    let NSApplication = Class::get("NSApplication").unwrap();
    unsafe {
        msg_send![super(app, NSApplication), finishLaunching];
    }
}

extern fn window_will_close(app: &mut Object, _sel: Sel, _notification: *mut Object) {
    window_map::with(|window_map| {
        let message_queue = window_map.get_mut(&ObjcObject(app as *mut _)).expect("No window existed in window map");
        message_queue.push_back(Message::Close);
    });
}

unsafe fn get_next_message_imp() -> extern fn (*mut Object, Sel, i64, *mut Object, *mut Object, BOOL) -> *mut Object {
    let NSApplication = Class::get("NSApplication").unwrap();
    let method = NSApplication.instance_method(sel!(nextEventMatchingMask:untilDate:inMode:dequeue:)).unwrap();
    ::std::mem::transmute(method.implementation())
}

/// Creates a window for the active application.
///
/// # Unsafety
///
/// - The `NSApplication` must fully initialized before attempting to open a window.
unsafe fn open_window(app: *mut Object) -> *mut Object {
    let NSWindow = Class::get("NSWindow").unwrap();
    let NSTrackingArea = Class::get("NSTrackingArea").unwrap();

    let point = NSPoint { x: 0.0, y: 0.0 };
    let size = NSSize { width: 500.0, height: 500.0 };
    let frame = NSRect { origin: point, size: size };

    let style_mask =
        NSWindowMask::NSTitledWindowMask as NSUInteger |
        NSWindowMask::NSClosableWindowMask as NSUInteger |
        NSWindowMask::NSMiniaturizableWindowMask as NSUInteger |
        NSWindowMask::NSResizableWindowMask as NSUInteger;

    // Create and initialize the window instance.
    let window: *mut Object = msg_send![NSWindow, alloc];
    let window: *mut Object = msg_send![
        window,
        initWithContentRect:frame
        styleMask:style_mask
        backing:NSBackingStoreType::NSBackingStoreRetained
        defer:NO
    ];

    // Create a view for the window.
    let frame = window.contentRectForFrameRect_(frame);
    let content = NSView::alloc(nil).initWithFrame_(frame);
    window.setContentView_(content);

    // Configure the window delegate.
    msg_send![window, setDelegate: app];

    // Ensure the window gets mouse move events.
    let result: BOOL = msg_send![window, makeFirstResponder: nil];
    println!("makeFirstResponder result: {:?}", result);
    msg_send![window, setAcceptsMouseMovedEvents: YES];

    let options: NSTrackingAreaOptions =
        NSTrackingActiveAlways |
        NSTrackingActiveAlways |
        NSTrackingMouseEnteredAndExited |
        NSTrackingMouseMoved;

    let view: *mut Object = msg_send![window, contentView];
    let bounds: NSRect = msg_send![view, bounds];
    let tracking_area: *mut Object = msg_send![NSTrackingArea, alloc];
    msg_send![
        tracking_area,
        initWithRect: bounds
        options: options
        owner: app
        userInfo: nil
    ];
    msg_send![view, addTrackingArea: tracking_area];

    // Show the window.
    msg_send![window, makeKeyAndOrderFront: app];

    window
}
