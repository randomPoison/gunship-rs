#![feature(std_misc)]

extern crate bootstrap_rs as bootstrap;
extern crate winapi;
extern crate kernel32;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;
use std::thread;
use std::ops::DerefMut;
use std::fs;

use bootstrap::time::Timer;
use bootstrap::window::Window;

const TARGET_FRAME_TIME_MS: f32 = 1.0 / 60.0 * 1000.0;

type EngineInit = fn (Box<Window>) -> Box<()>;
type EngineUpdateAndRender = fn (&mut ());
type EngineReload = fn (Box<()>) -> Box<()>;
type EngineClose = fn (&()) -> bool;

const SRC_LIB: &'static str = "gunship-ed06d2369a03ebbb.dll";
const LIB_PATH: &'static str = "gunship_temp.dll";

fn update_dll() {
    println!("remove file result: {:?}", fs::remove_file(LIB_PATH));
    println!("copy result: {:?}", fs::copy(SRC_LIB, LIB_PATH));
}

fn main() {
    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let mut window = Window::new("Gunship Game", instance);
    let window_address = window.deref_mut() as *mut Window;

    // Open the game as a dynamic library.
    let (mut _lib, mut engine, mut engine_update_and_render, mut engine_close) = {
        update_dll();
        let lib = DynamicLibrary::open(Some(Path::new(LIB_PATH))).unwrap();

        let engine_init = unsafe {
            mem::transmute::<*mut EngineInit, EngineInit>(lib.symbol("engine_init").unwrap())
        };

        let engine_update_and_render = unsafe {
            mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
        };

        let engine_close = unsafe {
            mem::transmute::<*mut EngineClose, EngineClose>(lib.symbol("engine_close").unwrap())
        };

        println!("calling engine_init()");
        let engine = engine_init(window);
        println!("done with engine_init()");

        (Some(lib), engine, engine_update_and_render, engine_close)
    };

    let timer = Timer::new();
    let mut reload_start = timer.now();
    loop {
        let start_time = timer.now();

        // Only reload every 5 seconds.
        if timer.elapsed(reload_start) > 5.0 {
            println!("time to reload library");

            _lib = None; // Drop the old dll so we can overwrite the file.

            update_dll();

            _lib = if let Ok(lib) = DynamicLibrary::open(Some(Path::new(LIB_PATH))) {
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

                Some(lib)
            } else {
                None
            };
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
