extern crate parse_obj;

use parse_obj::*;

static TRIANGLE_OBJ: &'static str = r#"
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 0.0 1.0 0.0

f 1// 2// 3//
"#;

fn main() {
    let obj = Obj::from_str(TRIANGLE_OBJ);
    println!("{:?}", obj);
}
