extern crate parse_collada;

use ::parse_collada::*;

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
