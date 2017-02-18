extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::*;

#[proc_macro_derive(ColladaElement, attributes(name, attribute, child))]
pub fn gen_(input: TokenStream) -> TokenStream {
    // Parse the string representation.
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();

    // Build the impl.
    match impl_hello_world(ast) {
        Ok(gen) => {
            let result = gen.parse().unwrap();
            result
        }
        Err(error) => { panic!("{}", error) }
    }
}

fn impl_hello_world(ast: DeriveInput) -> Result<quote::Tokens, String> {
    let type_name = &ast.ident;

    // Parse attributes on the top-level type.
    // ---------------------------------------
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

    let element_name = element_name.ok_or(r#"Type must have `#[name = "..."]` attribute when using `#[derive(ColladaElement)]`"#)?;

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
