extern crate parse_collada;

use parse_collada::*;

#[test]
fn no_xml_decl() {
    static DOCUMENT: &'static str = r#"
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset />
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn doctype() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <!DOCTYPE note SYSTEM "Note.dtd">
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset />
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn extra_whitespace() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>

    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">

        <asset          />

    </COLLADA      >

    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn collada_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset />
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(collada.version, "1.4.1");
}

#[test]
fn collada_missing_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema">
        <asset />
    </COLLADA>
    "#;

    let error = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(TextPosition { row: 2, column: 4 }, error.position());
    assert_eq!(&ErrorKind::MissingAttribute { element: "COLLADA".into(), attribute: "version".into() }, error.kind());
}

#[test]
fn collada_unexpected_attrib() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1" foo="bar">
        <asset />
    </COLLADA>
    "#;

    let error = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(TextPosition { row: 2, column: 4 }, error.position());
    assert_eq!(&ErrorKind::UnexpectedAttribute { element: "COLLADA".into(), attribute: "foo".into(), expected: COLLADA_ATTRIBS }, error.kind());
}

#[test]
fn collada_missing_asset() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1" foo="bar">
    </COLLADA>
    "#;

    let error = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(TextPosition { row: 2, column: 4 }, error.position());
    assert_eq!(&ErrorKind::UnexpectedAttribute { element: "COLLADA".into(), attribute: "foo".into(), expected: COLLADA_ATTRIBS }, error.kind());
}

#[test]
fn asset_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
        </asset>
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(Asset::default(), collada.asset);
}

#[test]
fn contributor_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
            </contributor>
        </asset>
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(vec![Contributor::default()], collada.asset.contributors);
}
