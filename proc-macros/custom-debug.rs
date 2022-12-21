use proc_macro2::{Ident, TokenStream};
use syn::{DeriveInput, Data, DataStruct, Fields, FieldsNamed, MetaNameValue, Lit, Field};
use quote::quote;

#[derive(Debug)]
struct Fd {
    attr_val: Option<String>,
    ident: Option<Ident>,
}
#[derive(Debug)]
pub struct DebugContext {
    name: Ident,
    fields: Vec<Fd>,
}

impl DebugContext {
    pub fn new (derive_input: &DeriveInput) -> Self {
        let name = &derive_input.ident;
        let fields = if let DeriveInput { data: Data::Struct(DataStruct{ fields: Fields::Named(FieldsNamed { named, ..}), ..}) ,..} = derive_input {
            named
        } else {
            panic!("can not get field")
        };

        let fds = fields.iter().map(|f| {
            let mut debug_value:Option<String> = None;
            if f.attrs.len() > 0 {
                debug_value = if let Ok(str) = get_debug_value(f) {
                    Some(str)
                } else {
                    None
                };
            }
            Fd {
                attr_val: debug_value,
                ident: f.ident.to_owned()
            }
        }).collect();
    
        DebugContext {
            name: name.to_owned(),
            fields: fds
        }
    }

    pub fn render (&self) -> TokenStream {
        let struct_name = &self.name;
        let fields = &self.fields;

        let format_strs: Vec<String> = fields.iter().map(|f| {
            if let Fd {attr_val: Some(d), .. } = f {
                return format!("{}: {}", f.ident.as_ref().unwrap(), d);
            } else {
                return format!("{}: {}", f.ident.as_ref().unwrap(), "{}");
            }
        }).collect();

        let format_v_key: Vec<&Ident> = fields.iter().map(|f| {
            return f.ident.as_ref().unwrap();
        }).collect();

        let format_str = format!("{} {{{{ {} }}}}", struct_name, format_strs.join(" "));

		quote! {
            impl std::fmt::Debug for #struct_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::write!(f, #format_str, #(self.#format_v_key),*)
                }
            }
		}
    }
}

fn get_debug_value(field: &Field) -> Result<String, &str> {
    if field.attrs.len() <= 0 {
        return Err("The field's attr is Empty");
    }
    match field.attrs[0].parse_meta() {
        Ok(syn::Meta::NameValue(MetaNameValue { lit: Lit::Str(lit_str), ..})) => {Ok(lit_str.value())},
        _ => Err("Not support meta type"),
    }
}