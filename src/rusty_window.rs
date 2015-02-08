#![feature(core, env)]

#[macro_use]
extern crate log;

extern crate winapi;

#[macro_use]
extern crate "rust-windows" as windows;

use std::ptr;

use winapi::{HBRUSH, CREATESTRUCTW};

use windows::main_window_loop;
use windows::instance::Instance;
use windows::resource::*;
use windows::window::{WindowImpl, Window, WndClass, WindowParams};
use windows::window::{OnCreate, OnSize, OnDestroy, OnPaint, OnFocus};
use windows::window;

static IDI_ICON: isize = 0x101;
static MENU_MAIN: isize = 0x201;

struct MainFrame
{
    win: Window
}

wnd_proc!(MainFrame, win, WM_CREATE, WM_DESTROY, WM_SIZE, WM_SETFOCUS, WM_PAINT);

impl OnCreate for MainFrame {
    fn on_create(&self, _cs: &CREATESTRUCTW) -> bool {
        true
    }
}

impl OnSize for MainFrame {
}

impl OnDestroy for MainFrame {}

impl OnPaint for MainFrame {
    fn on_paint(&self) {
    }
}

impl OnFocus for MainFrame {
    fn on_focus(&self, _w: Window) {
    }
}

impl MainFrame {
    fn new(instance: Instance, title: String) -> Option<Window> {
        let icon = Image::load_resource(instance, IDI_ICON, ImageType::IMAGE_ICON, 0, 0);
        let wnd_class = WndClass {
            classname: "MainFrame".to_string(),
            style: 0x0001 | 0x0002, // CS_HREDRAW | CS_VREDRAW
            icon: icon,
            icon_small: None,
            cursor: Image::load_cursor_resource(32514), // hourglass
            background: (5 + 1) as HBRUSH,
            menu: MenuResource::MenuId(MENU_MAIN),
            cls_extra: 0,
            wnd_extra: 0,
        };
        let res = wnd_class.register(instance);
        if !res {
            return None;
        }

        let wproc = Box::new(MainFrame {
            win: Window::null()
        });

        let win_params = WindowParams {
            window_name: title,
            style: window::WS_OVERLAPPEDWINDOW,
            x: 0,
            y: 0,
            width: 400,
            height: 400,
            parent: Window::null(),
            menu: ptr::null_mut(),
            ex_style: 0,
        };

        Window::new(instance,
                    Some(wproc as Box<WindowImpl + 'static>),
                    wnd_class.classname.as_slice(),
                    &win_params)
    }
}

fn main() {
    let instance = Instance::main_instance();
    let main = MainFrame::new(instance, "Rusty Window".to_string());
    let main = main.unwrap();

    main.show(1);
    main.update();

    let exit_code = main_window_loop();
    std::env::set_exit_status(exit_code as i32);
}