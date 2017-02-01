macro_rules! collada_element {
    ($element:expr, $type_name:ident => {}) => {
        #[derive(Debug, Clone, Default)]
        pub struct $type_name;

        impl ColladaElement for $type_name {
            fn parse(parser: &mut ColladaParser) -> Result<$type_name> {
                println!("Skippping over <{}>", $element);
                println!("WARNING: <{}> is not yet supported by parse-collada", $element);
                parser.skip_to_end_element($element);
                Ok($type_name)
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        contents: String,
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name(String);

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                let mut contents: Option<String> = None;

                loop {
                    let event = parser.next_event();
                    match event {
                        TextNode(text) => {
                            if contents.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from("contents"),
                                });
                            }

                            contents = Some(try!(parse_attrib::<String>(text)));
                        }

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                Ok($struct_name(contents.unwrap_or(String::new())))
            }
        }

        impl Deref for $struct_name {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        contents: $contents_type:ty,
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name($contents_type);

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                let mut contents: Option<$contents_type> = None;

                loop {
                    let event = parser.next_event();
                    match event {
                        TextNode(text) => {
                            if contents.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from("contents"),
                                });
                            }

                            contents = Some(try!(parse_attrib::<$contents_type>(text)));
                        }

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                if contents.is_none() {
                    return Err(Error::MissingTagContents(String::from($tag_name)));
                }

                Ok($struct_name(contents.unwrap() as $contents_type))
            }
        }

        impl Deref for $struct_name {
            type Target = $contents_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $struct_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    ($tag_name:expr, $struct_name:ident => {
        $(req attrib $req_attrib_str:expr => $req_attrib_name:ident: $req_attrib_type:ty,)*
        $(opt attrib $opt_attrib_str:expr => $opt_attrib_name:ident: $opt_attrib_type:ty,)*

        $(req child  $req_child_str:expr => $req_child_name:ident:  $req_child_type:ty,)*
        $(opt child  $opt_child_str:expr => $opt_child_name:ident:  $opt_child_type:ty,)*
        $(rep child  $rep_child_str:expr => $rep_child_name:ident:  $rep_child_type:ty,)*

        $(req enum $req_enum_name:ident: $req_enum_type:ident {
            $($req_var_tag:expr => $req_var_name:ident($req_var_type:ty),)*
        },)*
        $(opt enum $opt_enum_name:ident: $opt_enum_type:ident {
            $($opt_var_tag:expr => $opt_var_name:ident($opt_var_type:ty),)*
        },)*
        $(rep enum $rep_enum_name:ident: $rep_enum_type:ident {
            $($rep_var_tag:expr => $rep_var_name:ident($rep_var_type:ty),)*
        },)*

        $(contents: $contents_type:ty,)*
        $(opt contents: $opt_contents_type:ty,)*
    }) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name {
            $(pub $req_attrib_name: $req_attrib_type,)*
            $(pub $opt_attrib_name: Option<$opt_attrib_type>,)*

            $(pub $req_child_name:  $req_child_type,)*
            $(pub $opt_child_name:  Option<$opt_child_type>,)*
            $(pub $rep_child_name:  Vec<$rep_child_type>,)*

            $(pub $req_enum_name: $req_enum_type,)*
            $(pub $opt_enum_name: Option<$opt_enum_type>,)*
            $(pub $rep_enum_name: Vec<$rep_enum_type>,)*

            $(pub contents: $contents_type,)*
            $(pub contents: Option<$opt_contents_type>,)*
        }

        impl ColladaElement for $struct_name {
            fn parse(parser: &mut ColladaParser) -> Result<$struct_name> {
                $(let mut $req_attrib_name = None;)*
                $(let mut $opt_attrib_name = None;)*

                $(let mut $req_child_name = None;)*
                $(let mut $opt_child_name = None;)*
                $(let mut $rep_child_name = Vec::new();)*

                $(let mut $req_enum_name = None;)*
                $(let mut $opt_enum_name = None;)*
                $(let mut $rep_enum_name = Vec::new();)*

                $(let mut contents: Option<$contents_type> = None;)*
                $(let mut contents: Option<$opt_contents_type> = None;)*

                loop {
                    let event = parser.next_event();
                    match event {

                        // Required attributes.
                        $(Attribute($req_attrib_str, attrib_value) => {
                            if $req_attrib_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($req_attrib_name)),
                                });
                            }

                            let attrib = try!(parse_attrib(attrib_value));
                            $req_attrib_name = Some(attrib);
                        },)*

                        // Optional attributes.
                        $(Attribute($opt_attrib_str, attrib_value) => {
                            if $opt_attrib_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($opt_attrib_name)),
                                });
                            }

                            let attrib = try!(parse_attrib(attrib_value));
                            $opt_attrib_name = Some(attrib);
                        },)*

                        // Required children.
                        $(StartElement($req_child_str) => {
                            if $req_child_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($req_child_name)),
                                });
                            }

                            let child = try!(parse_element(parser));
                            $req_child_name = Some(child);
                        },)*

                        // Optional children.
                        $(StartElement($opt_child_str) => {
                            if $opt_child_name.is_some() {
                                return Err(Error::RepeatingChild {
                                    parent: String::from($tag_name),
                                    child: String::from(stringify!($opt_child_name)),
                                });
                            }

                            let child = try!(parse_element(parser));
                            $opt_child_name = Some(child);
                        },)*

                        // Repeating Children.
                        $(StartElement($rep_child_str) => {
                            let child = try!(parse_element(parser));
                            $rep_child_name.push(child);
                        },)*

                        // Required enum children.
                        $(
                            $(
                                StartElement($req_var_tag) => {
                                    if $req_enum_name.is_some() {
                                        return Err(Error::RepeatingChild {
                                            parent: String::from($tag_name),
                                            child: String::from(stringify!($req_enum_name)),
                                        });
                                    }

                                    let child: $req_var_type = try!(parse_element(parser));
                                    $req_enum_name = Some($req_enum_type::$req_var_name(child));
                                }
                            )*
                        )*

                        // Optional enum children.
                        $(
                            $(
                                StartElement($opt_var_tag) => {
                                    if $opt_enum_name.is_some() {
                                        return Err(Error::RepeatingChild {
                                            parent: String::from($tag_name),
                                            child: String::from(stringify!($opt_enum_name)),
                                        });
                                    }

                                    let child: $opt_var_type = try!(parse_element(parser));
                                    $opt_enum_name = Some($opt_enum_type::$opt_var_name(child));
                                }
                            )*
                        )*

                        // Repeating enum children.
                        $(
                            $(
                                StartElement($rep_var_tag) => {
                                    let child: $rep_var_type = try!(parse_element(parser));
                                    $rep_enum_name.push($rep_enum_type::$rep_var_name(child));
                                }
                            )*
                        )*

                        // Required text node.
                        $(
                            TextNode(text) => {
                                if contents.is_some() {
                                    return Err(Error::RepeatingChild {
                                        parent: String::from($tag_name),
                                        child: String::from("contents"),
                                    });
                                }

                                contents = Some(try!(parse_attrib::<$contents_type>(text)));
                            }
                        )*

                        // Optional text node.
                        $(
                            TextNode(text) => {
                                if contents.is_some() {
                                    return Err(Error::RepeatingChild {
                                        parent: String::from($tag_name),
                                        child: String::from("contents"),
                                    });
                                }

                                contents = Some(try!(parse_attrib::<$opt_contents_type>(text)));
                            }
                        )*

                        EndElement($tag_name) => break,
                        _ => return Err(illegal_event(event, $tag_name)),
                    }
                }

                $(
                    if $req_attrib_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_attrib_name)),
                        });
                    }
                )*

                $(
                    if $req_child_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_child_name)),
                        });
                    }
                )*

                $(
                    if $req_enum_name.is_none() {
                        return Err(Error::MissingRequiredChild {
                            parent: String::from($tag_name),
                            child: String::from(stringify!($req_enum_name)),
                        });
                    }
                )*

                $(
                    if contents.is_none() {
                        contents = Some(Default::default()) as Option<$contents_type>;
                    }
                )*

                Ok($struct_name {
                    $($req_attrib_name: $req_attrib_name.unwrap(),)*
                    $($opt_attrib_name: $opt_attrib_name,)*

                    $($req_child_name:  $req_child_name.unwrap(),)*
                    $($opt_child_name:  $opt_child_name,)*
                    $($rep_child_name:  $rep_child_name,)*

                    $($req_enum_name: $req_enum_name.unwrap(),)*
                    $($opt_enum_name: $opt_enum_name,)*
                    $($rep_enum_name: $rep_enum_name,)*

                    $(contents: contents.unwrap() as $contents_type,)*
                    $(contents: contents as Option<$opt_contents_type>,)*
                })
            }
        }
    };
}
