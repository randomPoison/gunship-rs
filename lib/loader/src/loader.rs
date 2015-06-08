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

type EngineInit = fn (Box<Window>) -> Engine;
type EngineUpdateAndRender = fn (&mut Engine);
type EngineReload = fn (Engine) -> Engine;
type EngineClose = fn (&Engine) -> bool;

fn main() {
    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let mut window = Window::new("Gunship Game", instance);
    let window_address = window.deref_mut() as *mut Window;

    // Open the game as a dynamic library.
    let (mut engine, mut engine_update_and_render, mut engine_close) = {
        let lib = DynamicLibrary::open(Some(Path::new("target/debug/deps/gunship-24517baeade73325.dll"))).unwrap();

        let engine_init = unsafe {
            mem::transmute::<*mut EngineInit, EngineInit>(lib.symbol("engine_init").unwrap())
        };

        let engine_update_and_render = unsafe {
            mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
        };

        let engine_close = unsafe {
            mem::transmute::<*mut EngineClose, EngineClose>(lib.symbol("engine_close").unwrap())
        };

        let engine = engine_init(window);

        (engine, engine_update_and_render, engine_close)
    };

    let timer = Timer::new();
    let mut reload_start = timer.now();
    loop {
        let start_time = timer.now();

        // Only reload every 5 seconds.
        if timer.elapsed(reload_start) > 5.0 {
            println!("time to reload library");

            if let Ok(lib) = DynamicLibrary::open(Some(Path::new("target/debug/deps/gunship-24517baeade73325.dll"))) {
                println!("reloading library");

                let engine_reload = unsafe {
                    mem::transmute::<*mut EngineReload, EngineReload>(lib.symbol("engine_reload").unwrap())
                };

                engine_update_and_render = unsafe {
                    mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
                };

                engine_close = unsafe {
                    mem::transmute::<*mut EngineClose, EngineClose>(lib.symbol("engine_close").unwrap())
                };

                engine = engine_reload(engine);

                reload_start = timer.now();
            }
        }

        unsafe {
            (&mut *window_address).handle_messages();
        }
        engine_update_and_render(&mut engine);
        if engine_close(&engine) {
            break;
        }

        // Wait for target frame time.
        let mut remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
        while remaining_time_ms > 1.0 {
            thread::sleep_ms(remaining_time_ms as u32);
            remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
        }

        while remaining_time_ms > 0.0 {
            remaining_time_ms = TARGET_FRAME_TIME_MS - timer.elapsed_ms(start_time);
        }
    }
}
