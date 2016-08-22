#![allow(non_snake_case)]

use macos::cocoa::appkit::*;
use macos::cocoa::foundation::*;
use macos::cocoa::base::{nil};
use objc::declare::*;
use objc::runtime::*;
use std::panic;
use window::Message;

pub struct Window {
    app: *mut Object,
    window: *mut Object,
}

unsafe fn finish_launching(app: &mut Object) {
    let NSApplication = Class::get("NSApplication").unwrap();
    let method = NSApplication.instance_method(sel!(finishLaunching)).unwrap();
    let imp: unsafe extern fn (*mut Object, Sel) = ::std::mem::transmute(method.implementation());
    imp(app, sel!(finishLaunching));
}

impl Window {
    pub fn new(_name: &str) -> Window {
        extern fn run(
            _self: &mut Object,
            _sel: Sel
        ) {
            // TODO: Create autorelease blocks?

            println!("run({:?}, {:?})", _self, _sel);

            unsafe {
                //println!("getting NSRunLoop and ensuring there's a default run loop");
                //let NSRunLoop = Class::get("NSRunLoop").unwrap();
                //let current_run_loop: *mut Object = msg_send![NSRunLoop, currentRunLoop];
                //println!("Current run loop: {:?}", current_run_loop);
                //println!("NSDefaultRunLoopMode: {:?}", NSDefaultRunLoopMode);

                println!("finishing launching");
                //msg_send![_super, finishLaunching];
                finish_launching(_self);
            }

            println!("Done with run()");
        }

        extern fn send_event(
            _self: &mut Object,
            _sel: Sel,
            event: *mut Object, // NSEvent
        ) {
            println!("send_event({:?})", event);

            unsafe {
                let _super: *mut Object = msg_send![_self, super];
                msg_send![_super, sendEvent: event];
            }
        }

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
                    run as extern fn (&mut Object, Sel));
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
        unsafe {
            //msg_send![app, run];
            msg_send![app,
                performSelectorOnMainThread: sel!(run)
                withObject: nil
                waitUntilDone: YES];

            let window = open_window();

            msg_send![pool, release];

            Window {
                app: app,
                window: window,
            }
        }
    }

    pub fn close(&self) {
        // unsafe { self.window.performClose_(nil); }
    }

    pub fn next_message(&mut self) -> Option<Message> {
        println!("getting message");

        unsafe {
            // TODO: Create autorelease blocks?

            let NSDate = Class::get("NSDate").unwrap();
            let distant_future = msg_send![NSDate, distantFuture];

            //let event = msg_send![
            //    self.app,
            //    nextEventMatchingMask: NSAnyEventMask
            //    untilDate: distant_future
            //    inMode: 0
            //    dequeue: YES];
            let imp = get_next_message_imp();

            let event = imp(
                self.app,
                sel!(nextEventMatchingMask:untilDate:inMode:dequeue:),
                0xffffffff, //NSAnyEventMask,
                distant_future,
                NSDefaultRunLoopMode,
                YES,
            );

            let type_ptr: NSEventType = msg_send![event, type];
            println!("event: {:?}", type_ptr);

            msg_send![self.app, sendEvent:event];
            msg_send![self.app, updateWindows];
        }

        None
    }

    pub fn get_rect(&self) -> (i32, i32, i32, i32) {
        (0, 0, 1, 1)
    }
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
unsafe fn open_window() -> *mut Object {
    println!("Opening an NSWindow");

    let NSWindow = Class::get("NSWindow").unwrap();

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

    // Show the window.
    msg_send![
        window,
        makeKeyAndOrderFront:nil
    ];
    msg_send![window, orderFrontRegardless];

    // window.makeKeyAndOrderFront_(nil);
    // window.orderFrontRegardless();

    println!("Done opening NSWindow");

    window
}
