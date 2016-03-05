extern crate parse_obj;

use parse_obj::*;

static TRIANGLE_OBJ: &'static str = r#"
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 0.0 1.0 0.0

vn 0.0 0.0 1.0

f 0/0 1/0 2/0
"#;

fn main() {
    let obj = Obj::from_str(TRIANGLE_OBJ);
    println!("{:?}", obj);
}