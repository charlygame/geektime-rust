use proc_macro::{TokenStream };
use syn::{DeriveInput, parse_macro_input, Data, DataEnum, Ident };

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let clone_input = input.clone();
    let derive_input = parse_macro_input!(input as DeriveInput);
    println!(">> derive_input: {:#?}", derive_input);
    let fields = match &derive_input.data {
        Data::Enum( DataEnum { variants, .. }) => variants,
        _ => panic!("expected enum or match expression"),
        
    };

    let idents = fields.iter().map(|field| &field.ident).collect::<Vec<_>>();
    let mut prev_ident: Option<&Ident> = None;
    println!("{:#?}", idents);
    for ident in idents.into_iter() {
        if let Some(prev) = prev_ident {
            if ident < prev {
                panic!("{} should sort before {}", ident, prev);
            }
        }
        prev_ident = Some(ident)
    }
    clone_input
}
