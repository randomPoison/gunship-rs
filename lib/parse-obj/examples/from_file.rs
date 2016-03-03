extern crate parse_obj;

use parse_obj::*;

fn main() {
    let obj = Obj::from_file("examples/epps_head.obj");
    println!("{:?}", obj);
}
