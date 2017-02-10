use {AnyUri, DateTime, Result, Error, ErrorKind, Unit, UpAxis, UTC, utils};
use std::io::Read;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::EventReader;
use xml::reader::XmlEvent::*;

/// The logic behind parsing the COLLADA document.
///
/// `from_str()` and `read()` just create the `xml::EventReader` and then defer to `parse()`.
pub fn parse<R: Read>(mut reader: EventReader<R>, version: String, base: Option<AnyUri>) -> Result<Collada> {
    // The next event must be the `<asset>` tag. No text data is allowed, and
    // whitespace/comments aren't emitted.
    let (_name, _, _) = utils::required_start_element(&mut reader, "COLLADA", "asset")?;
    let asset = parse_asset(&mut reader)?;

    // Eat any events until we get to the `</COLLADA>` tag.
    // TODO: Actually parse the body of the document.
    loop {
        match reader.next()? {
            EndElement { ref name } if name.local_name == "COLLADA" => { break }
            _ => {}
        }
    }

    // TODO: Verify the next event is the `EndDocument` event.
    match reader.next()? {
        EndDocument => {}

        // Same logic here as with the starting event. The only things that can come after the
        // close tag are comments, white space, and processing instructions, all of which we
        // ignore. This can change with future versions of xml-rs, though.
        event @ _ => { panic!("Unexpected event: {:?}", event); }
    }

    Ok(Collada {
        version: version,
        asset: asset,
        base_uri: base,
    })
}

fn parse_asset<R: Read>(reader: &mut EventReader<R>) -> Result<Asset> {
    unimplemented!()
}

fn parse_contributor<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Contributor> {
    // Make sure the `<contributor>` element has no attributes.
    if attributes.len() != 0 {
        return Err(Error {
            position: reader.position(),
            kind: ErrorKind::UnexpectedAttribute {
                element: "contributor".into(),
                attribute: attributes[0].name.local_name.clone(),
                expected: vec![],
            },
        })
    }

    let mut contributor = Contributor::default();

    static EXPECTED_ELEMENTS: &'static [&'static str] = &[
        "author",
        "author_email",
        "author_website",
        "authoring_tool",
        "comments",
        "copyright",
        "source_data",
    ];

    let mut current_element = 0;
    while let Some((element_name, element_attributes, _)) = utils::optional_start_element(reader, "contributor", EXPECTED_ELEMENTS, current_element)? {
        match &*element_name.local_name {
            "author" => {
                utils::verify_attributes(reader, "author", element_attributes)?;
                contributor.author = utils::text_only_element(reader, "author")?;
            }

            "author_email" => {
                utils::verify_attributes(reader, "author_email", element_attributes)?;
                contributor.author_email = utils::text_only_element(reader, "author_email")?;
            }

            "author_website" => {
                utils::verify_attributes(reader, "author_website", element_attributes)?;
                contributor.author_website = utils::text_only_element(reader, "author_website")?.map(Into::into);
            }

            "authoring_tool" => {
                utils::verify_attributes(reader, "authoring_tool", element_attributes)?;
                contributor.authoring_tool = utils::text_only_element(reader, "authoring_tool")?;
            }

            "comments" => {
                utils::verify_attributes(reader, "comments", element_attributes)?;
                contributor.comments = utils::text_only_element(reader, "authoring_tool")?;
            }

            "copyright" => {
                utils::verify_attributes(reader, "copyright", element_attributes)?;
                contributor.copyright = utils::text_only_element(reader, "copyright")?;
            }

            "source_data" => {
                utils::verify_attributes(reader, "source_data", element_attributes)?;
                contributor.source_data = utils::text_only_element(reader, "source_data")?.map(Into::into);
            }

            _ => { panic!("Unexpected element name: {}", element_name); }
        }

        current_element = EXPECTED_ELEMENTS
            .iter()
            .position(|&name| name == element_name.local_name)
            .expect("Element wasn't in expected elements");
    }

    Ok(contributor)
}

/// Represents a parsed COLLADA document.
#[derive(Debug, Clone, PartialEq)]
pub struct Collada {
    /// The version string for the COLLADA specification used by the document.
    ///
    /// Only "1.4.0", "1.4.1", and "1.5.0" are supported currently.
    pub version: String,

    /// The base uri for any relative URIs in the document.
    ///
    /// Specified by the `base` attribute on the root `<COLLADA>` element.
    pub base_uri: Option<AnyUri>,

    /// Global metadata about the COLLADA document.
    pub asset: Asset,
}

impl Collada {
    /// Read a COLLADA document from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use parse_collada::Collada;
    ///
    /// static DOCUMENT: &'static str = r#"
    ///     <?xml version="1.0" encoding="utf-8"?>
    ///     <COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
    ///         <asset />
    ///     </COLLADA>
    /// "#;
    ///
    /// let collada = Collada::from_str(DOCUMENT).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn from_str(source: &str) -> Result<Collada> {
        let reader = EventReader::new_with_config(source.as_bytes(), utils::PARSER_CONFIG.clone());
        utils::parse(reader)
    }

    /// Attempts to parse the contents of a COLLADA document.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_variables)]
    /// use std::fs::File;
    /// use parse_collada::Collada;
    ///
    /// let file = File::open("resources/blender_cube.dae").unwrap();
    /// let collada = Collada::read(file).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the document is invalid or malformed in some way. For details about
    /// COLLADA versions, 3rd party extensions, and any other details that could influence how
    /// a document is parsed see the [crate-level documentation][crate].
    ///
    /// [crate]: index.html
    pub fn read<R: Read>(reader: R) -> Result<Collada> {
        let reader = EventReader::new_with_config(reader, utils::PARSER_CONFIG.clone());
        utils::parse(reader)
    }
}

/// Asset-management information about an element.
///
/// Includes both asset metadata, such as a list of contributors and keywords, as well
/// as functional information, such as units of distance and the up axis for the asset.
///
/// # COLLADA Versions
///
/// `coverage` and `extras` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    /// The list of contributors who worked on the asset.
    pub contributors: Vec<Contributor>,
    pub coverage: Option<GeographicLocation>,
    pub created: DateTime<UTC>,
    pub keywords: Option<String>,
    pub modified: DateTime<UTC>,
    pub revision: Option<String>,
    pub subject: Option<String>,
    pub title: Option<String>,
    pub unit: Unit,
    pub up_axis: UpAxis,
    pub extras: Vec<Extra>,
}

/// Information about a contributor to an asset.
///
/// Contributor data is largely free-form text data meant to informally describe either the author
/// or the author's work on the asset. The exceptions are `author_email`, `author_website`, and
/// `source_data`, which are strictly formatted data (be it a URI or email address).
///
/// # COLLADA Versions
///
/// `author_email` and `author_website` were added in COLLADA version `1.5.0`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contributor {
    /// The author's name, if present.
    pub author: Option<String>,

    /// The author's full email address, if present.
    // TODO: Should we use some `Email` type? The 1.5.0 COLLADA spec provides an RFC defining the
    // exact format this data follows (I assume it's just the RFC that defines valid email
    // addresses).
    pub author_email: Option<String>,

    /// The URL for the author's website, if present.
    pub author_website: Option<AnyUri>,

    /// The name of the authoring tool.
    pub authoring_tool: Option<String>,

    /// Free-form comments from the author.
    pub comments: Option<String>,

    /// Copyright information about the asset. Does not adhere to a formatting standard.
    pub copyright: Option<String>,

    /// A URI reference to the source data for the asset.
    ///
    /// For example, if the asset based off a file `tank.s3d`, the value might be
    /// `c:/models/tank.s3d`.
    pub source_data: Option<AnyUri>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeographicLocation {
    latitude: f64,
    longitude: f64,
    mode: Altitude,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Altitude {
    Absolute(f64),
    RelativeToGround(f64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extra;
