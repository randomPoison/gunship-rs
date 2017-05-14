extern crate parse_collada;

use ::parse_collada::*;

#[test]
fn collada_asset_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;


    let expected = Collada {
        version: "1.4.1".into(),
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
fn collada_missing_asset() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
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
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor />
            <contributor />
            <contributor />
            <created>2017-02-07T20:44:30Z</created>
            <keywords>foo bar baz</keywords>
            <modified>2017-02-07T20:44:30Z</modified>
            <revision>7</revision>
            <subject>A thing</subject>
            <title>Model of a thing</title>
            <unit meter="7" name="septimeter" />
            <up_axis>Z_UP</up_axis>
        </asset>
    </COLLADA>
    "#;

    let expected = Asset {
        contributors: vec![Contributor::default(), Contributor::default(), Contributor::default()],
        coverage: None,
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
        extras: Vec::default(),
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, collada.asset);
}

#[test]
fn asset_blender() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>Blender User</author>
                <authoring_tool>Blender 2.78.0 commit date:2016-10-24, commit time:12:20, hash:e8299c8</authoring_tool>
            </contributor>
            <created>2017-02-01T09:29:54</created>
            <modified>2017-02-01T09:29:54</modified>
            <unit name="meter" meter="1"/>
            <up_axis>Z_UP</up_axis>
        </asset>
    </COLLADA>
    "#;

    let expected = Asset {
        contributors: vec![
            Contributor {
                author: Some("Blender User".into()),
                authoring_tool: Some("Blender 2.78.0 commit date:2016-10-24, commit time:12:20, hash:e8299c8".into()),
                .. Contributor::default()
            },
        ],
        coverage: None,
        created: "2017-02-01T09:29:54".parse().unwrap(),
        keywords: Vec::new(),
        modified: "2017-02-01T09:29:54".parse().unwrap(),
        revision: None,
        subject: None,
        title: None,
        unit: Unit {
            meter: 1.0,
            name: "meter".into(),
        },
        up_axis: UpAxis::Z,
        extras: Vec::default(),
    };

    let collada = Collada::from_str(DOCUMENT).unwrap();
    assert_eq!(expected, collada.asset);
}

#[test]
fn asset_wrong_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
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
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 12 },
        kind: ErrorKind::UnexpectedElement {
            parent: "asset",
            element: "coverage".into(),
            expected: vec!["contributor", "created", "keywords", "modified", "revision", "subject", "title", "unit", "up_axis"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_minimal() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
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
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
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
        author_email: None,
        author_website: None,
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
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <comments>This is a sample COLLADA document.</comments>
                <authoring_tool>Atom</authoring_tool>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 7, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "authoring_tool".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_child() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments>This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
                <foo>Some foo data</foo>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
        </asset>
    </COLLADA>
    "#;

    let expected = Error {
        position: TextPosition { row: 10, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "foo".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_wrong_version() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <author_email>dl@email.com</author_email>
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

    let expected = Error {
        position: TextPosition { row: 6, column: 16 },
        kind: ErrorKind::UnexpectedElement {
            parent: "contributor".into(),
            element: "author_email".into(),
            expected: vec!["author", "authoring_tool", "comments", "copyright", "source_data"],
        },
    };

    let actual = Collada::from_str(DOCUMENT).unwrap_err();
    assert_eq!(expected, actual);
}

#[test]
fn contributor_illegal_attribute() {
    static DOCUMENT: &'static str = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor foo="bar">
                <author>David LeGare</author>
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
    <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
        <asset>
            <contributor>
                <author>David LeGare</author>
                <authoring_tool>Atom</authoring_tool>
                <comments foo="bar">This is a sample COLLADA document.</comments>
                <copyright>David LeGare, free for public use</copyright>
                <source_data>C:/models/tank.s3d</source_data>
            </contributor>
            <created>2017-02-07T20:44:30Z</created>
            <modified>2017-02-07T20:44:30Z</modified>
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
