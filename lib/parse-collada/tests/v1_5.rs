extern crate parse_collada;

use ::parse_collada::*;

#[test]
fn collada_asset_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;


    let expected = Collada {
        version: "1.5.0".into(),
        base_uri: None,
        asset: Asset {
            contributors: vec![],
            coverage: None,
            created: "2017-02-07T20:44:30Z".parse().unwrap(),
            keywords: Vec::new(),
            modified: "2017-02-07T20:44:30Z".parse().unwrap(),
            revision: None,
            subject: None,
            title: None,
            unit: Unit {
                meter: 1.0,
                name: "meter".into(),
            },
            up_axis: UpAxis::Y,
            extras: vec![],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn collada_missing_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 2, column: 4 },
        kind: ErrorKind::MissingAttribute {
            element: "COLLADA".into(),
            attribute: "version".into()
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn collada_unexpected_attrib() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0" foo="bar">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 2, column: 4 },
        kind: ErrorKind::UnexpectedAttribute {
            element: "COLLADA".into(),
            attribute: "foo".into(),
            expected: vec!["version", "xmlns", "base"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn collada_missing_asset() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 3, column: 4 },
        kind: ErrorKind::MissingElement {
            parent: "COLLADA".into(),
            expected: "asset",
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn asset_full() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor />
            <contributor />
            <contributor />
            <coverage>
                <geographic_location>
                    <longitude>-105.2830</longitude>
                    <latitude>40.0170</latitude>
                    <altitude mode="relativeToGround">0</altitude>
                </geographic_location>
            </coverage>
            <created>2017-02-07T20:44:30Z</created>
            <keywords>foo bar baz</keywords>
            <modified>2017-02-07T20:44:30Z</modified>
            <revision>7</revision>
            <subject>A thing</subject>
            <title>Model of a thing</title>
            <unit meter="7" name="septimeter" />
            <up_axis>Z_UP</up_axis>
            <extra />
            <extra />
        </asset>
    </COLLADA>
    "#;

    let expected = Asset {
        contributors: vec![Contributor::default(), Contributor::default(), Contributor::default()],
        coverage: Some(GeographicLocation {
            longitude: -105.2830,
            latitude: 40.0170,
            altitude: Altitude::RelativeToGround(0.0),
        }),
        created: "2017-02-07T20:44:30Z".parse().unwrap(),
        keywords: vec!["foo".into(), "bar".into(), "baz".into()],
        modified: "2017-02-07T20:44:30Z".parse().unwrap(),
        revision: Some("7".into()),
        subject: Some("A thing".into()),
        title: Some("Model of a thing".into()),
        unit: Unit {
            meter: 7.0,
            name: "septimeter".into(),
        },
        up_axis: UpAxis::Z,
        extras: vec![Extra::default(), Extra::default()],
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, collada.asset);
}

#[test]
fn contributor_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor />
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(vec![Contributor::default()], collada.asset.contributors);
}

#[test]
fn contributor_full() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <author_email>dl@email.com</author_email>
                <author_website>david.com</author_website>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Contributor {
        author: Some("David LeGare".into()),
        author_email: Some("dl@email.com".into()),
        author_website: Some("david.com".parse().unwrap()),
        authoring_tool: Some("Atom".into()),
        comments: Some("This is a sample COLLADA document.".into()),
        copyright: Some("David LeGare, free for public use".into()),
        source_data: Some("C:/models/tank.s3d".parse().unwrap()),
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(vec![expected], collada.asset.contributors);
}

#[test]
fn contributor_wrong_order() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <comments>This is a sample COLLADA document.</comments>
                <authoring_tool>Atom</authoring_tool>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "authoring_tool".into(),
            expected: vec!["author", "author_email", "author_website", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_child() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
                <foo>Some foo data</foo>
            </contributor>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 10, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "foo".into(),
            expected: vec!["author", "author_email", "author_website", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_attribute() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor foo="bar">
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 4, column: 12 },
        kind: ErrorKind::UnexpectedAttribute {
            element: "contributor".into(),
            attribute: "foo".into(),
            expected: vec![],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_child_attribute() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments foo="bar">This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 16 },
        kind: ErrorKind::UnexpectedAttribute {
            element: "comments".into(),
            attribute: "foo".into(),
            expected: vec![],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn extra_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
            <extra>
                <technique profile="cool" />
            </extra>
        </asset>
    </COLLADA>
    "#;

    let expected = Extra {
        id: None,
        name: None,
        type_hint: None,
        asset: None,
        techniques: vec![Technique { profile: "cool".into(), xmlns: None, data: Vec::default() }],
    };

    let actual = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, actual.asset.extras[0]);
}

#[test]
fn extra_full() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.5.0">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
            <extra id="myExtra" name="Betty" type="extra">
                <asset>
                    <created>2017-02-07T20:44:30Z</created>
                    <modified>2017-02-07T20:44:30Z</modified>
                </asset>
                <technique profile="foo" />
                <technique profile="bar" />
                <technique profile="baz" />
            </extra>
        </asset>
    </COLLADA>
    "#;

    let expected = Extra {
        id: Some("myExtra".into()),
        name: Some("Betty".into()),
        type_hint: Some("extra".into()),
        asset: Some(Asset {
            contributors: vec![],
            coverage: None,
            created: "2017-02-07T20:44:30Z".parse().unwrap(),
            keywords: Vec::new(),
            modified: "2017-02-07T20:44:30Z".parse().unwrap(),
            revision: None,
            subject: None,
            title: None,
            unit: Unit::default(),
            up_axis: UpAxis::default(),
            extras: Vec::default(),
        }),
        techniques: vec![
            Technique { profile: "foo".into(), xmlns: None, data: Vec::default() },
            Technique { profile: "bar".into(), xmlns: None, data: Vec::default() },
            Technique { profile: "baz".into(), xmlns: None, data: Vec::default() },
        ],
    };

    let actual = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, actual.asset.extras[0]);
}
