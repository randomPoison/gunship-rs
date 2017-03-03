extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::{Tokens, ToTokens};
use syn::*;

#[proc_macro_derive(ColladaElement, attributes(name, attribute, child))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the string representation.
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();

    // Build the impl.
    match generate_impl(ast) {
        Ok(gen) => {
            let result = gen.parse().unwrap();
            result
        }
        Err(error) => { panic!("{}", error) }
    }
}

fn process_derive_input(input: DeriveInput) -> Result<ElementConfiguration, String> {
    let struct_ident = input.ident;

    // Process the top-level attributes on the type to find the `#[name = "foo"]` attribute.
    // -------------------------------------------------------------------------------------
    let element_name = {
        let mut element_name = None;

        for attribute in input.attrs {
            match attribute.value {
                MetaItem::NameValue(attr_name, Lit::Str(value, _)) => {
                    if attr_name == "name" {
                        element_name = Some(value);
                    }
                }

                _ => {}
            }
        }

        element_name.ok_or(r#"Type must have `#[name = "..."]` attribute when using `#[derive(ColladaElement)]`"#)?
    };

    // Process the body of the type and gather information about attributes and children.
    // ----------------------------------------------------------------------------------
    let mut children = Vec::new();
    let mut attributes = Vec::new();

    let fields = match input.body {
        Body::Enum(_) => { return Err("`#[derive(ColladaElement)]` does not support enum types")?; }

        Body::Struct(VariantData::Struct(fields)) => { fields }

        Body::Struct(VariantData::Tuple(_)) => { return Err("`#[derive(ColladaElement)]` does not support tuple structs")?; }

        Body::Struct(VariantData::Unit) => { Vec::new() }
    };

    for field in fields {
        // We only support struct-structs, so all fields will have an ident.
        let member_name = field.ident.unwrap();

        // Determine the data type and occurrences for the member.
        let mut path = match field.ty.clone() {
            Ty::Path(None, path) => { path }
            _ => { return Err("`#[derive(ColladaElement)]` doesn't support this member type")?; }
        };

        // Determine the number of occurrences based on the declared type:
        //
        // - `Option<T>` is optional with inner type `T`.
        // - `Vec<T>` is repeating with inner type `T`.
        // - Everything else is required with inner type as declared.
        let segment = path.segments.pop().expect("Somehow got an empty path ?_?");

        // We only support angle bracket parameters (because we're only looking for `Option<T>`
        // and `Vec<T>`), so extract the parameter data and throw away all others.
        let parameter_data = match segment.parameters {
            PathParameters::AngleBracketed(param) => { param }
            _ => { return Err("Round brace function parameters are not supported")?; }
        };

        // Depending on the number of parameters (0 or 1) we determine the occurrences and the
        // type of the actual data.
        let (occurrences, inner_type) = if parameter_data.types.len() == 0 {
            // No type parameters, so we're not looking at `Option<T>` or `Vec<T>`. That means the
            // child is required and that the field's type is the type of the child data.
            (ChildOccurrences::Required, field.ty)
        } else {
            // There's 1 type parameter, so determine if we're looking at an `Option<T>`, which
            // means optional occurrences of a `T`, or a `Vec<T>`, which means optional many
            // occurrences of `T`.
            let inner_type = parameter_data.types[0].clone();
            match segment.ident.as_ref() {
                "Option" => { (ChildOccurrences::Optional, inner_type) }
                "Vec" => { (ChildOccurrences::OptionalMany, inner_type) }
                _ => { return Err("Unexpected child type with parameters, only `Vec<T>` and `Option<T>` are allowed to have type parameters")?; }
            }
        };

        // Determine the data type of the inner type, e.g. if it's `String` or another ColladaElement.
        let data_type = match inner_type {
            Ty::Path(None, mut path) => {
                let segment = path.segments.pop().expect("Somehow got an empty path ?_?");
                if segment.ident.as_ref() == "String" {
                    DataType::String
                } else {
                    DataType::ColladaElement(segment.ident)
                }
            },

            _ => { return Err("`#[derive(ColladaElement)]` doesn't support this member type")?; }
        };

        // Determine whether we're looking at a child or an attribute based on whether the member
        // has a `#[child]` or an `#[attribute]` attribute.
        for attribute in field.attrs {
            match attribute.name() {
                "child" => {
                    children.push(Child {
                        member_name: member_name.clone(),
                        element_name: member_name.to_string(),
                        occurrences: occurrences,
                        ty: data_type,
                    });
                    break;
                }

                "attribute" => {
                    let occurrences = match occurrences {
                        ChildOccurrences::Optional => AttributeOccurrences::Optional,
                        ChildOccurrences::Required => AttributeOccurrences::Required,

                        ChildOccurrences::OptionalMany | ChildOccurrences::RequiredMany => {
                            return Err("Attribute may not be repeating, meaning it may not be of type `Vec<T>`".into());
                        }
                    };

                    attributes.push(Attribute {
                        member_name: member_name.clone(),
                        attrib_name: member_name.to_string(),
                        occurrences: occurrences,
                    });
                    break;
                }

                _ => {}
            }
        }
    }

    Ok(ElementConfiguration {
        struct_ident: struct_ident,
        element_name: element_name,
        attributes: attributes,
        children: children,
    })
}

struct ElementConfiguration {
    struct_ident: Ident,
    element_name: String,
    attributes: Vec<Attribute>,
    children: Vec<Child>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttributeOccurrences {
    Optional,
    Required,
}

struct Attribute {
    member_name: Ident,
    attrib_name: String,
    occurrences: AttributeOccurrences,
}

enum DataType {
    String,
    ColladaElement(Ident),
}

struct Child {
    member_name: Ident,
    element_name: String,
    occurrences: ChildOccurrences,
    ty: DataType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildOccurrences {
    Optional,
    Required,
    OptionalMany,
    RequiredMany,
}

impl ToTokens for ChildOccurrences {
    fn to_tokens(&self, tokens: &mut Tokens) {
        match *self {
            ChildOccurrences::Optional => { tokens.append("Optional"); }

            ChildOccurrences::Required => { tokens.append("Required"); }

            ChildOccurrences::OptionalMany => { tokens.append("Many"); }

            ChildOccurrences::RequiredMany => { tokens.append("RequiredMany"); }
        }
    }
}

fn generate_impl(derive_input: DeriveInput) -> Result<quote::Tokens, String> {
    let ElementConfiguration { struct_ident, element_name, attributes, children } = process_derive_input(derive_input)?;

    // Generate declarations for the member variables of the struct.
    // -------------------------------------------------------------
    let member_decls = {
        let attribs = attributes.iter()
            .map(|attrib| {
                let ident = &attrib.member_name;
                quote! { let mut #ident = None; }
            });
        let childs = children.iter()
            .map(|child| {
                let ident = &child.member_name;
                quote! { let mut #ident = None; }
            });

        quote! {
            #( #attribs )*
            #( #childs )*
        }
    };

    // Generate code for parsing attributes.
    // -------------------------------------
    let attributes_impl = {
        let matches = attributes.iter()
            .map(|attrib| {
                let &Attribute { ref member_name, ref attrib_name, .. } = attrib;
                quote! {
                    #attrib_name => { #member_name = Some(attribute.value.parse()?); }
                }
            });

        let attrib_names = attributes.iter()
            .map(|attrib| { &*attrib.attrib_name });

        let required_attribs = attributes.iter()
            .filter(|attrib| { attrib.occurrences == AttributeOccurrences::Required })
            .map(|attrib| {
                let &Attribute { ref member_name, ref attrib_name, .. } = attrib;
                quote! {
                    let #member_name = #member_name.ok_or(Error {
                        position: reader.position(),
                        kind: ErrorKind::MissingAttribute {
                            element: #element_name,
                            attribute: #attrib_name,
                        },
                    })?;
                }
            });

        quote! {
            for attribute in attributes {
                match &*attribute.name.local_name {
                    #( #matches )*

                    attrib_name @ _ => {
                        return Err(Error {
                            position: reader.position(),
                            kind: ErrorKind::UnexpectedAttribute {
                                element: #element_name,
                                attribute: attrib_name.into(),
                                expected: vec![ #( #attrib_names ),* ],
                            },
                        })
                    }
                }
            }

            #( #required_attribs )*
        }
    };

    // Generate code for parsing children.
    // -----------------------------------
    let children_impl = {
        let decls = children.iter()
            .map(|child| {
                let &Child { ref member_name, ref element_name, occurrences, ref ty } = child;

                let handle_result = match (occurrences, ty) {
                    (ChildOccurrences::Optional, &DataType::String) => {
                        quote! {
                            #member_name = utils::optional_text_contents(reader, #element_name)?;
                        }
                    }

                    (ChildOccurrences::Required, &DataType::String) => {
                        quote! {
                            let result = utils::required_text_contents(reader, #element_name)?;
                            #member_name = Some(result);
                        }
                    }

                    (ChildOccurrences::OptionalMany, &DataType::String) => {
                        quote! {
                            if let Some(result) = utils::optional_text_contents(reader, #element_name)? {
                                #member_name.push(result);
                            }
                        }
                    }

                    (ChildOccurrences::RequiredMany, &DataType::String) => {
                        quote! {
                            if let Some(result) = utils::optional_text_contents(reader, #element_name)? {
                                #member_name.push(result);
                            }
                        }
                    }

                    (ChildOccurrences::Optional, &DataType::ColladaElement(ref ident)) => {
                        quote! {
                            let result = #ident::parse_element()?;
                            #member_name = Some(result);
                        }
                    }

                    (ChildOccurrences::Required, &DataType::ColladaElement(ref ident)) => {
                        quote! {
                            let result = #ident::parse_element()?;
                            #member_name = Some(result);
                        }
                    }

                    (ChildOccurrences::OptionalMany, &DataType::ColladaElement(ref ident)) => {
                        quote! {
                            let result = #ident::parse_element()?;
                            #member_name.push(result);
                        }
                    }

                    (ChildOccurrences::RequiredMany, &DataType::ColladaElement(ref ident)) => {
                        quote! {
                            let result = #ident::parse_element()?;
                            #member_name.push(result);
                        }
                    }
                };

                quote! {
                    ChildConfiguration {
                        name: #element_name,
                        occurrences: #occurrences,

                        action: &mut |reader, attributes| {
                            #handle_result
                            Ok(())
                        },
                    }
                }
            });

        quote! {
            ElementConfiguration {
                name: #element_name,
                children: &mut [
                    #( #decls ),*
                ],
            }.parse_children(reader)?;
        }
    };

    // Generate code to construct final result.
    // ----------------------------------------
    let result_decl = {
        let attribs = attributes.iter()
            .map(|attrib| {
                let ident = &attrib.member_name;
                quote! { #ident: #ident }
            });
        let childs = children.iter()
            .map(|child| {
                let ident = &child.member_name;
                quote! { #ident: #ident }
            });

        quote! {
            Ok(#struct_ident {
                #( #attribs ),*
                #( #childs ),*
            })
        }
    };

    // Put all the pieces together.
    // ----------------------------
    Ok(quote! {
        impl ColladaElement for #struct_ident {
            fn parse_element<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Self> {
                // use utils;
                // use utils::{ElementConfiguration, ChildConfiguration};

                #member_decls

                #attributes_impl

                #children_impl

                #result_decl
            }

            fn name() -> &'static str {
                #element_name
            }
        }
    })
}
