#![feature(std_misc, fs_time)]

extern crate bootstrap_rs as bootstrap;
extern crate winapi;
extern crate kernel32;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;
use std::thread;
use std::fs::{self, File};
use std::rc::Rc;
use std::cell::RefCell;

use bootstrap::time::Timer;
use bootstrap::window::Window;

const TARGET_FRAME_TIME_MS: f32 = 1.0 / 60.0 * 1000.0;

type EngineInit = fn (Rc<RefCell<Window>>) -> Box<()>;
type EngineReload = fn (&()) -> Box<()>;
type EngineUpdateAndRender = fn (&mut ());
type EngineClose = fn (&()) -> bool;
type EngineDrop = fn(Box<()>);

const SRC_LIB: &'static str = "gunship-ed06d2369a03ebbb.dll";

fn update_dll(dest: &str, last_modified: &mut u64) -> bool {
    if let Ok(file) = File::open(SRC_LIB) {
        let metadata = file.metadata().unwrap();
        let modified = metadata.modified();

        if modified > *last_modified {
            println!("copy result: {:?}", fs::copy(SRC_LIB, dest));
            *last_modified = modified;
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn load_engine_procs(lib: &DynamicLibrary) -> (EngineUpdateAndRender, EngineClose, EngineDrop) {
    let engine_update_and_render = unsafe {
        mem::transmute::<*mut EngineUpdateAndRender, EngineUpdateAndRender>(lib.symbol("engine_update_and_render").unwrap())
    };

    let engine_close = unsafe {
        mem::transmute::<*mut EngineClose, EngineClose>(lib.symbol("engine_close").unwrap())
    };

    let engine_drop = unsafe {
        mem::transmute::<*mut EngineDrop, EngineDrop>(lib.symbol("engine_drop").unwrap())
    };

    (engine_update_and_render, engine_close, engine_drop)
}

/// # TODO
///
/// - Keep track of the temp files made and then delete them when done running.
/// - Support reloading game code.
/// - Reload the windows message proc when the engine is reloaded.
fn main() {
    let mut counter = 0..;

    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let window = Window::new("Gunship Game", instance);
    let mut temp_paths: Vec<String> = Vec::new();

    // Open the game as a dynamic library.
    let mut last_modified = 0;
    let (mut _lib, mut engine, mut engine_update_and_render, mut engine_close, mut engine_drop) = {
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap().to_string());
        update_dll(&lib_path, &mut last_modified);
        let lib = DynamicLibrary::open(Some(Path::new(&lib_path))).unwrap();
        temp_paths.push(lib_path);

        let engine_init = unsafe {
            mem::transmute::<*mut EngineInit, EngineInit>(lib.symbol("engine_init").unwrap())
        };

        let (engine_update_and_render, engine_close, engine_drop) = load_engine_procs(&lib);

        let engine = engine_init(window.clone());

        (lib, engine, engine_update_and_render, engine_close, engine_drop)
    };

    let timer = Timer::new();
    loop {
        let start_time = timer.now();

        // Only reload if file has changed.
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap());
        if update_dll(&lib_path, &mut last_modified) {
            if let Ok(lib) = DynamicLibrary::open(Some(Path::new(&lib_path))) {

                let engine_reload = unsafe {
                    mem::transmute::<*mut EngineReload, EngineReload>(lib.symbol("engine_reload").unwrap())
                };

                let new_engine = engine_reload(&engine);
                engine_drop(engine);

                engine = new_engine;

                // Load procs from the new lib.
                let procs = load_engine_procs(&lib);
                engine_update_and_render = procs.0;
                engine_close = procs.1;
                engine_drop = procs.2;

                // Drop the old dll and stash the new one to keep it loaded.
                _lib = lib;
            }
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
