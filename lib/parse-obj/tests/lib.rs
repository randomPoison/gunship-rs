extern crate parse_obj;

use parse_obj::*;

static TRIANGLE_OBJ: &'static str = r#"
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 0.0 1.0 0.0

f 1// 2// 3//
"#;

static TRIANGLE_WITH_NORM: &'static str = r#"
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 0.0 1.0 0.0

vt 1.0 1.0 1.0

vn 1.0 0.0 0.0

f 1/1/1 2/1/1 3/1/1
"#;

#[test]
fn test_iterator() {
    {
        let obj = Obj::from_str(TRIANGLE_OBJ).unwrap();

        let mut face_iter = obj.faces();
        let mut face = face_iter.next().unwrap();
        assert!(face_iter.next().is_none());

        assert_eq!(Some(((-1.0, -1.0, 0.0, 1.0), None, None)), face.next());
        assert_eq!(Some(((1.0, -1.0, 0.0, 1.0), None, None)), face.next());
        assert_eq!(Some(((0.0, 1.0, 0.0, 1.0), None, None)), face.next());
    }

    {
        let obj = Obj::from_str(TRIANGLE_WITH_NORM).unwrap();

        let mut face_iter = obj.faces();
        let mut face = face_iter.next().unwrap();
        assert!(face_iter.next().is_none());

        assert_eq!(Some(((-1.0, -1.0, 0.0, 1.0), Some((1.0, 1.0, 1.0)), Some((1.0, 0.0, 0.0)))), face.next());
        assert_eq!(Some(((1.0, -1.0, 0.0, 1.0), Some((1.0, 1.0, 1.0)), Some((1.0, 0.0, 0.0)))), face.next());
        assert_eq!(Some(((0.0, 1.0, 0.0, 1.0), Some((1.0, 1.0, 1.0)), Some((1.0, 0.0, 0.0)))), face.next());
    }
}
