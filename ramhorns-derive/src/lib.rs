// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

//! <img src="https://raw.githubusercontent.com/maciejhirsz/ramhorns/master/ramhorns.svg?sanitize=true" alt="Ramhorns logo" width="250" align="right" style="background: #fff; margin: 0 0 1em 1em;">
//!
//! ## Ramhorns
//!
//! This is a `#[derive]` macro crate, [for documentation go to main crate](https://docs.rs/ramhorns).

// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]

extern crate proc_macro;

use quote::quote;
use proc_macro::TokenStream;
use syn::{ItemStruct, Field, Fields, Type};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use fnv::FnvHasher;

use std::hash::Hasher;

type UnitFields = Punctuated<Field, Comma>;

#[proc_macro_derive(Context)]
pub fn logos(input: TokenStream) -> TokenStream {
    let item: ItemStruct = syn::parse(input).expect("#[derive(Context)] can be only applied to structs");

    // panic!("{:#?}", item);

    let name = &item.ident;
    let generics = &item.generics;
    let unit_fields = UnitFields::new();

    let fields = match &item.fields {
        Fields::Named(fields) => fields.named.iter(),
        Fields::Unnamed(fields) => fields.unnamed.iter(),
        _ => unit_fields.iter(),
    };

    let mut fields = fields
        .enumerate()
        .map(|(index, field)| {
            let (name, token) = field.ident
                .as_ref()
                .map(|ident| (ident.to_string(), quote!(#ident)))
                .unwrap_or_else(|| {
                    use syn::{LitInt, IntSuffix};
                    use proc_macro2::Span;

                    let lit = LitInt::new(index as u64, IntSuffix::None, Span::call_site());

                    (index.to_string(), quote!(#lit))
                });

            let mut hasher = FnvHasher::default();

            hasher.write(name.as_bytes());

            let hash = hasher.finish();

            (name, token, hash, deref_str(&field.ty))
        })
        .collect::<Vec<_>>();

    fields.sort_unstable_by(|a, b| (a.2).cmp(&b.2));

    let render_field_escaped = fields.iter().filter_map(|(_, field, hash, deref)| {
        deref.as_ref().map(|deref| {
            quote! {
                #hash => encoder.write_escaped(#deref self.#field),
            }
        })
    });

    let render_field_unescaped = fields.iter().filter_map(|(_, field, hash, deref)| {
        deref.as_ref().map(|deref| {
            quote! {
                #hash => encoder.write(#deref self.#field),
            }
        })
    });

    let render_field_section = fields.iter().map(|(_, field, hash, _)| {
        quote! {
            #hash => self.#field.render_section(section, encoder),
        }
    });

    let render_field_inverse = fields.iter().map(|(_, field, hash, _)| {
        quote! {
            #hash => self.#field.render_inverse(section, encoder),
        }
    });

    let fields = fields.iter().filter_map(|(_, field, _, deref)| deref.as_ref().map(|_| field));

    // FIXME: decouple lifetimes from actual generics with trait boundaries
    let tokens = quote! {
        impl#generics ramhorns::Context for #name#generics {
            fn capacity_hint(&self, tpl: &ramhorns::Template) -> usize {
                tpl.capacity_hint() #( + self.#fields.len() )*
            }

            fn render_field_escaped<W: std::io::Write>(&self, hash: u64, encoder: &mut ramhorns::Encoder<W>) -> std::io::Result<()> {
                match hash {
                    #( #render_field_escaped )*
                    _ => Ok(())
                }
            }

            fn render_field_unescaped<W: std::io::Write>(&self, hash: u64, encoder: &mut ramhorns::Encoder<W>) -> std::io::Result<()> {
                match hash {
                    #( #render_field_unescaped )*
                    _ => Ok(())
                }
            }

            fn render_field_section<'section, W: std::io::Write>(&self, hash: u64, section: ramhorns::Section<'section>, encoder: &mut ramhorns::Encoder<W>) -> std::io::Result<()> {
                match hash {
                    #( #render_field_section )*
                    _ => Ok(())
                }
            }

            fn render_field_inverse<'section, W: std::io::Write>(&self, hash: u64, section: ramhorns::Section<'section>, encoder: &mut ramhorns::Encoder<W>) -> std::io::Result<()> {
                match hash {
                    #( #render_field_inverse )*
                    _ => Ok(())
                }
            }
        }
    };

    // panic!("{}", tokens);

    TokenStream::from(tokens).into()
}

fn deref_str(mut ty: &Type) -> Option<proc_macro2::TokenStream> {
    let mut refs = -1i32;

    while let Type::Reference(r) = ty {
        refs += 1;
        ty = &*r.elem;
    }

    match quote!(#ty).to_string().as_str() {
        "str" | "String" => {},
        _ => return None
    }

    match refs {
        4 => Some(quote!(***)),
        3 => Some(quote!(**)),
        1 => Some(quote!(*)),
        0 => Some(quote!()),
        -1 => Some(quote!(&)),
        _ => None,
    }
}
