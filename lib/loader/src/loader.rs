#![feature(std_misc)]

extern crate gunship;

use std::dynamic_lib::DynamicLibrary;
use std::path::Path;
use std::mem;

use gunship::*;

fn main() {
    // Open the game as a dynamic library;
    let lib = DynamicLibrary::open(Some(Path::new("gunship.dll"))).unwrap();
    println!("successfully loaded game library");

    let engine_init = unsafe {
        mem::transmute::<*mut extern fn () -> Engine, extern fn () -> Engine>(lib.symbol("engine_init").unwrap())
    };
    println!("successfully loaded init function");

    println!("calling engine_init()");
    let mut engine = engine_init();
    engine.main_loop();
}
