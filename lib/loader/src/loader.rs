#![feature(std_misc)]

extern crate gunship;
extern crate bootstrap_rs as bootstrap;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;
use std::thread;
use std::ops::DerefMut;

use bootstrap::time::Timer;
use bootstrap::window::Window;

use gunship::*;
use gunship::engine::TARGET_FRAME_TIME_MS;

type EngineInit = extern fn (Box<Window>) -> Engine;
type EngineUpdateAndRender = extern fn (&mut Engine);

fn main() {
    // Open the game as a dynamic library;
    let lib = DynamicLibrary::open(Some(Path::new("gunship-24517baeade73325.dll"))).unwrap();
    println!("successfully loaded game library");

    let engine_init = unsafe {
        mem::transmute::<*mut EngineInit, EngineInit>(lib.symbol("engine_init").unwrap())
    };
    println!("successfully loaded init function");

    let engine_update_and_render = unsafe {
        mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
    };

    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let mut window = Window::new("Gunship Game", instance);
    let window_address = window.deref_mut() as *mut Window;
    // let renderer = gl_render::init(&window);

    println!("calling engine_init()");
    let mut engine = engine_init(window);
    let timer = Timer::new();
    loop {
        let start_time = timer.now();

        // TODO: Try reloading the dll.

        unsafe {
            (&mut *window_address).handle_messages();
        }
        engine_update_and_render(&mut engine);
        if engine.close() {
            break;
        }

        // Wait for target frame time.
        let mut remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
        while remaining_time_ms > 1.0 {
            remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
            thread::sleep_ms(remaining_time_ms as u32);
        }

        while remaining_time_ms > 0.0 {
            remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
        }
    }
}
