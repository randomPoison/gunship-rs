#![feature(dynamic_lib)]
#![allow(deprecated)]

extern crate bootstrap_rs as bootstrap;
extern crate winapi;
extern crate kernel32;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;
use std::thread;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

use bootstrap::time::Timer;
use bootstrap::window::Window;
use bootstrap::windows::file::file_modified;

const TARGET_FRAME_TIME_MS: f32 = 1.0 / 60.0 * 1000.0;

type EngineInit = fn (Rc<RefCell<Window>>) -> Box<()>;
type EngineReload = fn (&()) -> Box<()>;
type EngineUpdateAndRender = fn (&mut ());
type EngineClose = fn (&()) -> bool;
type EngineDrop = fn(Box<()>);
type GameInit = fn(&mut ());
type GameReload = fn(&(), &());

fn update_dll(src_lib: &str, dest: &str, last_modified: &mut u64) -> bool {
    let modified = match file_modified(src_lib) {
        Ok(modified) => modified,
        Err(_) => return false,
    };
    if modified > *last_modified {
        println!("copy result: {:?}", fs::copy(src_lib, dest));
        *last_modified = modified;
        true
    } else {
        false
    }
}

fn load_engine_procs(lib: &DynamicLibrary) -> (EngineUpdateAndRender, EngineClose, EngineDrop) {
    let engine_update_and_render = unsafe {
        let symbol = lib.symbol("engine_update_and_render").unwrap();
        mem::transmute::<*mut u8, EngineUpdateAndRender>(symbol)
    };

    let engine_close = unsafe {
        let symbol = lib.symbol("engine_close").unwrap();
        mem::transmute::<*mut u8, EngineClose>(symbol)
    };

    let engine_drop = unsafe {
        let symbol = lib.symbol("engine_drop").unwrap();
        mem::transmute::<*mut u8, EngineDrop>(symbol)
    };

    (engine_update_and_render, engine_close, engine_drop)
}

/// # TODO
///
/// - Keep track of the temp files made and then delete them when done running.
/// - Support reloading game code.
/// - Reload the windows message proc when the engine is reloaded.
pub fn run_loader(src_lib: &str) {
    let mut counter = 0..;

    // Statically create a window and load the renderer for the engine.
    let instance = bootstrap::init();
    let window = Window::new("Gunship Game", instance);
    let mut temp_paths: Vec<String> = Vec::new();


    // Open the game as a dynamic library.
    let mut last_modified = 0;
    let (mut _lib, mut engine, mut engine_update_and_render, mut engine_close, mut engine_drop) = {
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap().to_string());
        if !update_dll(src_lib, &lib_path, &mut last_modified) {
            panic!("Unable to find library {} for dynamic loading", src_lib);
        }
        let lib = match DynamicLibrary::open(Some(Path::new(&lib_path))) {
            Ok(lib) => lib,
            Err(error) => panic!("Unable to open DLL {} with error: {}", lib_path, error),
        };
        temp_paths.push(lib_path);

        let engine_init = unsafe {
            let symbol = lib.symbol("engine_init").unwrap();
            mem::transmute::<*mut u8, EngineInit>(symbol)
        };

        let game_init = unsafe {
            let symbol = lib.symbol("game_init").unwrap();
            mem::transmute::<*mut u8, GameInit>(symbol)
        };

        let (engine_update_and_render, engine_close, engine_drop) = load_engine_procs(&lib);

        let mut engine = engine_init(window.clone());
        game_init(&mut engine);

        (lib, engine, engine_update_and_render, engine_close, engine_drop)
    };

    let timer = Timer::new();
    loop {
        let start_time = timer.now();

        // Only reload if file has changed.
        let lib_path = format!("gunship_lib_{}.dll", counter.next().unwrap());
        if update_dll(src_lib, &lib_path, &mut last_modified) {
            if let Ok(lib) = DynamicLibrary::open(Some(Path::new(&lib_path))) {

                let engine_reload = unsafe {
                    let symbol = lib.symbol("engine_reload").unwrap();
                    mem::transmute::<*mut u8, EngineReload>(symbol)
                };

                let game_reload = unsafe {
                    let symbol = lib.symbol("game_reload").unwrap();
                    mem::transmute::<*mut u8, GameReload>(symbol)
                };

                let mut new_engine = engine_reload(&engine);
                game_reload(&engine, &mut new_engine);
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
