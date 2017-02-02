extern crate parse_collada;

use parse_collada::*;

#[test]
fn collada_element() {
    static DOCUMENT: &'static str =
r#"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
</COLLADA>
"#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn collada_element_whitespace() {
    static DOCUMENT: &'static str =
r#"
<?xml version="1.0" encoding="utf-8"?>

<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">

</COLLADA>

"#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn collada_element_missing_version() {
    static DOCUMENT: &'static str =
r#"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema">
</COLLADA>
"#;

    let error = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(TextPosition { row: 1, column: 0 }, error.position());
    assert_eq!(&ErrorKind::MissingAttribute { element: "COLLADA".into(), attribute: "version".into() }, error.kind());
}

#[test]
fn collada_element_unexpected_attrib() {
    static DOCUMENT: &'static str =
r#"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1" foo="bar">
</COLLADA>
"#;

    let error = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(TextPosition { row: 1, column: 0 }, error.position());
    assert_eq!(&ErrorKind::UnexpectedAttribute { element: "COLLADA".into(), attribute: "foo".into(), expected: COLLADA_ATTRIBS, is_duplicate: false }, error.kind());
}
