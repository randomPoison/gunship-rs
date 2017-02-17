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
        Ok(gen) => { gen.parse().unwrap() }
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
        quote! {
            unimplemented!();
        }
    };

    // Put all the pieces together.
    // ----------------------------
    Ok(quote! {
        impl ColladaElement for #type_name {
            fn parse<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Self> {
                #attributes_impl

                #children_impl
            }
        }
    })
}
