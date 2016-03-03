extern crate parse_obj;

use parse_obj::*;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::open("examples/epps_head.obj").unwrap();
    let mut text = String::new();

    file.read_to_string(&mut text).unwrap();

    let obj = Obj::from_str(&text);
    println!("{:?}", obj);
}
