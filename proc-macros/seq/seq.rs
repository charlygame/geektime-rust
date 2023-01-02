use std::fmt::{Debug};
use proc_macro2::{Span};
use syn::{ PatRange, Ident, parse::{ParseStream, Parse, Result}, Pat, Token, braced, Attribute, RangeLimits, ExprLit, parenthesized };
use quote::{quote, ToTokens};


trait Render {
    fn render(&self, range: &Range) -> proc_macro2::TokenStream;
}

#[derive(Debug)]
struct Range {
    start: i32,
    end: i32,
    limit: RangeLimits,
}

impl Range {
    pub fn new (pat: Pat)-> Result<Range> {
        if let Pat::Range( PatRange { limits, lo, hi, .. }, ..) = pat {
            let start;
            let end;
            
            match lo.as_ref() {
                syn::Expr::Lit(ExprLit {lit: syn::Lit::Int(int), ..}, ..) => start = int.base10_parse::<i32>(),
                _ => todo!(),
            }
            
            match hi.as_ref() {
                syn::Expr::Lit(ExprLit {lit: syn::Lit::Int(int), ..}, ..) => end = int.base10_parse::<i32>(),
                _ => todo!(),
            }

            Ok(Range {
                start: start?,
                end: end?,
                limit: limits
            })
        } else {
            panic!("Range parse fail ")
        }
    }
}

#[derive(Debug)]
struct RepStat {
    ident: Ident,
}

impl RepStat {
    pub fn new(input: ParseStream)-> Result<Self> {
        input.parse::<Token![#]>()?;
        let content;
        parenthesized!(content in input);
        input.parse::<Token![*]>()?;

        let ident: Ident = content.parse()?;
        content.parse::<Token![~]>()?;
        let _ = content.parse::<Ident>()?;
        content.parse::<Token![,]>()?;
        Ok(RepStat {
            ident
        })
    }
}

#[derive(Debug)]
struct EnumData {
    ident: Ident,
    field_prefix: Ident,
    attrs: Vec<Attribute>,
}
impl EnumData {
    pub fn set_attrs(&mut self, attrs: Vec<Attribute>) {
        self.attrs = attrs;
    }
}

impl Parse for EnumData {
    fn parse(input: ParseStream) -> Result<Self> {
        let _ = input.parse::<Token![enum]>()?;
        let ident = input.parse::<Ident>()?;
        let enum_body_buf;
        braced!(enum_body_buf in input);
        let rep_stat = RepStat::new(&enum_body_buf)?;
        Ok(EnumData {
            ident,
            field_prefix: rep_stat.ident,
            attrs: [].into()
        })
    }
}

impl Render for EnumData {
    fn render(&self, range: &Range) -> proc_macro2::TokenStream {
        let mut fields: Vec<Ident> =[].to_vec();
        let field_prefix = &self.field_prefix;
        let Range {limit, start, end} = range;
        match limit {
            RangeLimits::HalfOpen(_) => {
                fields = (start.to_owned() ..end.to_owned()).map(|i| {
                    Ident::new(&format!("{}{}",&field_prefix.to_string(), i), Span::call_site())
                }).collect();
            },
            RangeLimits::Closed(_) => {
                fields = (start.to_owned()..=end.to_owned()).map(|i| {
                    Ident::new( &format!("{}{}",&field_prefix.to_string(), i), Span::call_site())
                }).collect();
            },
        }
        
        let attr_tokens:Vec<proc_macro2::TokenStream> = self.attrs.iter().map(|a| {
           a.into_token_stream()
        }).collect();
        let enum_ident = &self.ident;

        quote!(
            #(#attr_tokens)*
            enum #enum_ident {
               #(#fields,)*
            }
        )
    }
}


pub struct Seq {
    range: Range,
    data: Box<dyn Render>,
}
impl Seq {
    pub fn render(&self) -> proc_macro2::TokenStream {
        self.data.render(&self.range)
    }
}

impl Parse for Seq{
    fn parse(input: ParseStream) -> Result<Self> {
        let _: Ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let pat: Pat = input.parse()?;

        let body_buf;
        let _ = braced!(body_buf in input);
        let attrs = body_buf.call(Attribute::parse_outer)?;
        let lookahead = body_buf.lookahead1();
        if lookahead.peek(Token![enum]) {
            let mut enum_data = body_buf.parse::<EnumData>()?;
            enum_data.set_attrs(attrs);
            Ok(Seq {
                range: Range::new(pat)?,
                data: Box::new(enum_data),
            })
        } else {
           panic!("not support type")
        }        
    }
}
