extern crate parse_collada;

use ::parse_collada::*;

#[test]
fn load_document() {
    static TEST_DOCUMENT: &'static [u8] = include_bytes!("../resources/blender_cube.dae");

    let document = String::from_utf8(TEST_DOCUMENT.into()).unwrap();
    let _ = Collada::from_str(&*document).unwrap();
}

#[test]
fn no_xml_decl() {
    static DOCUMENT: &'static str = r#"
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let _ = Collada::from_str(DOCUMENT).unwrap();
}

#[test]
fn doctype() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <!DOCTYPE note SYSTEM "Note.dtd">
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let _ = Collada::from_str(DOCUMENT).unwrap();
}

#[test]
fn extra_whitespace() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>

    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">

        <asset        >
            <created>    2017-02-07T20:44:30Z        </created       >
            <modified    > 2017-02-07T20:44:30Z             </modified      >
        </asset>

    </COLLADA      >

    "#;

    let _ = Collada::from_str(DOCUMENT).unwrap();
}
