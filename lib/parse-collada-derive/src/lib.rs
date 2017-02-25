extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
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

        for attribute in ast.attrs {
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
        let path = match field.ty {
            Ty::Path(None, path) => { path }
            _ => { return Err("`#[derive(ColladaElement)]` doesn't support this member type")?; }
        };

        // Determine the number of occurrences based on the declared type:
        //
        // - `Option<T>` is optional with inner type `T`.
        // - `Vec<T>` is repeating with inner type `T`.
        // - Everything else is required with inner type as declared.
        let segment = path.segments[0].clone();

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

        // Determine whether we're looking at a child or an attribute based on whether the member
        // has a `#[child]` or an `#[attribute]` attribute.
        unimplemented!();
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

enum AttributeOccurrences {
    RequiredOptional,
}

struct Attribute {
    member_name: Ident,
    attrib_name: String,
    occurrences: AttributeOccurrences,
    ty: DataType,
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

enum ChildOccurrences {
    Optional,
    Required,
    OptionalMany,
    RequiredMany,
}

fn generate_impl(derive_input: DeriveInput) -> Result<quote::Tokens, String> {
    let ElementConfiguration { struct_ident, element_name, children, attributes } = process_derive_input(derive_input)?;

    // Generate declarations for the member variables of the struct.
    // -------------------------------------------------------------
    let member_decls = {
        match ast.body {
            Body::Enum(_) => { panic!("`#[derive(ColladaElement)]` does not yet support enum types"); }

            Body::Struct(VariantData::Unit) => { quote! {} }

            Body::Struct(VariantData::Tuple(_)) => { panic!("`#[derive(ColladaElement)]` does not yet support tuple structs"); }

            Body::Struct(VariantData::Struct(ref fields)) => {
                let decls = fields
                    .iter()
                    .map(|field| {
                        let ident = field.ident.as_ref().unwrap();
                        quote! { let mut #ident = None; }
                    });

                quote! {
                    #( #decls )*
                }
            }
        }
    };

    // Generate code for parsing attributes.
    // -------------------------------------
    let attributes_impl = {
        // TODO: Actualy look at the members and find out what attributes we should be looking for.
        // For now we'll return a default impl that assumes there should be no attributes.
        quote! {
            ::utils::verify_attributes(reader, #element_name, attributes)?;
        }
    };

    // Generate code for parsing children.
    // -----------------------------------
    let children_impl = {
        match ast.body {
            Body::Enum(_) => { panic!("`#[derive(ColladaElement)]` does not yet support enum types"); }

            Body::Struct(VariantData::Unit) => { quote! {} }

            Body::Struct(VariantData::Tuple(_)) => { panic!("`#[derive(ColladaElement)]` does not yet support tuple structs"); }

            Body::Struct(VariantData::Struct(ref fields)) => {
                let decls = fields.iter()
                    .filter_map(|field| {
                        // Determine whether the element is a child or attribute based on the
                        // attributes for the field. Here we only care about children.
                        for attribute in &field.attrs {
                            // Determine the name of the child element based on the attributes on
                            // the element.
                            let child_element_name = match attribute.value {
                                // Children declared as `#[child]`.
                                MetaItem::Word(ref ident) => {
                                    if ident == "child" {
                                        field.ident.clone().unwrap().to_string()
                                    } else {
                                        continue;
                                    }
                                }

                                // Children declared as `#[child(element = "foo")]`
                                MetaItem::List(..) => {
                                    unimplemented!()
                                }

                                MetaItem::NameValue(..) => { continue; }
                            };

                            let field_ident = field.ident.clone().unwrap();

                            let occurrences = {
                                // TODO: Determine child occurrences based on type:
                                //
                                // - `Option<T>` is optional.
                                // - `Vec<T>` is many.
                                // - `Vec<T>` with `#[required]` attribute is required many.
                                // - All others are required.
                                quote! { Optional }
                            };

                            let child_action = {
                                // TODO: Determine child action based on type. Specific types (e.g.
                                // `String`) get special handling, but anything that implements
                                // `ColladaElement` will use that impl.
                                quote! {
                                    utils::verify_attributes(reader, #child_element_name, attributes)?;
                                    #field_ident = utils::optional_text_contents(reader, #child_element_name).map(Into::into)?;
                                    Ok(())
                                }
                            };

                            return Some(quote! {
                                ChildConfiguration {
                                    name: #child_element_name,
                                    occurrences: #occurrences,

                                    action: &mut |reader, attributes| {
                                        #child_action
                                    },
                                }
                            });
                        }

                        None
                    },
                );

                quote! {
                    ElementConfiguration {
                        name: #element_name,
                        children: &mut [
                            #(
                                #decls
                            ),*
                        ],
                    }.parse_children(reader)?;
                }
            }
        }
    };

    // Generate code to construct final result.
    // ----------------------------------------
    let result_decl = {
        match ast.body {
            Body::Enum(_) => { panic!("`#[derive(ColladaElement)]` does not yet support enum types"); }

            Body::Struct(VariantData::Unit) => { quote! {} }

            Body::Struct(VariantData::Tuple(_)) => { panic!("`#[derive(ColladaElement)]` does not yet support tuple structs"); }

            Body::Struct(VariantData::Struct(ref fields)) => {
                let decls = fields
                    .iter()
                    .map(|field| {
                        let ident = field.ident.as_ref().unwrap();
                        quote! { #ident: #ident }
                    });

                quote! {
                    Ok(#type_name {
                        #( #decls ),*
                    })
                }
            }
        }
    };

    // Put all the pieces together.
    // ----------------------------
    Ok(quote! {
        impl ColladaElement for #type_name {
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
