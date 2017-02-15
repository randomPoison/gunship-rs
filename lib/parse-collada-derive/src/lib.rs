extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(ColladaElement, attributes(attribute, child))]
pub fn gen_(input: TokenStream) -> TokenStream {
    // Parse the string representation.
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();

    // Build the impl.
    let gen = impl_hello_world(&ast);

    // Return the generated impl.
    gen.parse().unwrap()
}

fn impl_hello_world(ast: &DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl ColladaElement for #name {
            fn parse<R: Read>(reader: &mut EventReader<R>, attributes: Vec<OwnedAttribute>) -> Result<Self> {
                unimplemented!()
            }
        }
    }
}
