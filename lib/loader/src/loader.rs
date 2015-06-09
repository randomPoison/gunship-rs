#![feature(std_misc, fs_time)]

extern crate bootstrap_rs as bootstrap;
extern crate winapi;
extern crate kernel32;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;
use std::thread;
use std::ops::DerefMut;
use std::fs::{self, File};

use bootstrap::time::Timer;
use bootstrap::window::Window;

const TARGET_FRAME_TIME_MS: f32 = 1.0 / 60.0 * 1000.0;

type EngineInit = fn (Box<Window>) -> Box<()>;
type EngineUpdateAndRender = fn (&mut ());
type EngineReload = fn (Box<()>) -> Box<()>;
type EngineClose = fn (&()) -> bool;

const SRC_LIB: &'static str = "gunship-ed06d2369a03ebbb.dll";

fn update_dll(dest: &str, last_modified: &mut u64) -> bool {
    if let Ok(file) = File::open(SRC_LIB) {
        println!("Getting file metadata");
        let metadata = file.metadata().unwrap();
        let modified = metadata.modified();
        println!("last_modified: {}, modified: {}", last_modified, modified);

        if modified > *last_modified {
            println!("file has been updated");
            println!("copy result: {:?}", fs::copy(SRC_LIB, dest));
            *last_modified = modified;
            true
        } else {
            println!("file is the same");
            false
        }
    } else {
        println!("couldn't open file info");
        false
    }
}

/// # TODO
///
/// - Copy the complete game runtime into the new DLL's memory space when reloading, then have the old DLL clean
///   up the old data before releasing it.
/// - Keep track of the temp files made and then delete them when done running.
fn main() {
    let mut counter = 0..;

    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let mut window = Window::new("Gunship Game", instance);
    let window_address = window.deref_mut() as *mut Window;

    // Open the game as a dynamic library.
    let mut last_modified = 0;
    let (mut _lib, mut engine, mut engine_update_and_render, mut engine_close) = {
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap().to_string());
        update_dll(&lib_path, &mut last_modified);
        let lib = DynamicLibrary::open(Some(Path::new(&lib_path))).unwrap();

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
    loop {
        let start_time = timer.now();

        // Only reload if file has changed.
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap());
        if update_dll(&lib_path, &mut last_modified) {
            println!("reloading library");
            if let Ok(lib) = DynamicLibrary::open(Some(Path::new(&lib_path))) {
                println!("opened library");

                let engine_reload = unsafe {
                    mem::transmute::<*mut EngineReload, EngineReload>(lib.symbol("engine_reload").unwrap())
                };

                engine_update_and_render = unsafe {
                    mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
                };

                engine_close = unsafe {
                    mem::transmute::<*mut EngineClose, EngineClose>(lib.symbol("engine_close").unwrap())
                };

                println!("calling engine_reload()");
                engine = engine_reload(engine);
                println!("done with engine_reload()");

                // Drop the old dll and load the new one.
                _lib = Some(lib);
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
