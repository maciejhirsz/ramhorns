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
use syn::{ItemStruct, Field, Fields};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use fnv::FnvHasher;

use std::hash::Hasher;

type UnitFields = Punctuated<Field, Comma>;

#[proc_macro_derive(Content, attributes(md))]
pub fn content_derive(input: TokenStream) -> TokenStream {
    let item: ItemStruct = syn::parse(input).expect("#[derive(Content)] can be only applied to structs");

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
            let iter = field.attrs
                .iter()
                .filter_map(|attr| attr.path.segments.first())
                .map(|pair| &pair.into_value().ident);

            let mut method = None;

            for attr in iter {
                if attr == "md" {
                    method = Some(quote!(render_cmark));
                }
            }

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

            (name, token, hash, method)
        })
        .collect::<Vec<_>>();

    fields.sort_unstable_by(|a, b| (a.2).cmp(&b.2));

    let render_escaped = quote!(render_escaped);
    let render_field_escaped = fields.iter().map(|(_, field, hash, method)| {
        let method = method.as_ref().unwrap_or(&render_escaped);

        quote! {
            #hash => self.#field.#method(encoder).map(|_| true),
        }
    });

    let render_unescaped = quote!(render_unescaped);
    let render_field_unescaped = fields.iter().map(|(_, field, hash, method)| {
        let method = method.as_ref().unwrap_or(&render_unescaped);

        quote! {
            #hash => self.#field.#method(encoder).map(|_| true),
        }
    });

    let render_field_section = fields.iter().map(|(_, field, hash, _)| {
        quote! {
            #hash => self.#field.render_section(section, encoder).map(|_| true),
        }
    });

    let render_field_inverse = fields.iter().map(|(_, field, hash, _)| {
        quote! {
            #hash => self.#field.render_inverse(section, encoder).map(|_| true),
        }
    });

    let fields = fields.iter().map(|(_, field, _, _)| field);

    // FIXME: decouple lifetimes from actual generics with trait boundaries
    let tokens = quote! {
        impl#generics ramhorns::Content for #name#generics {
            fn capacity_hint(&self, tpl: &ramhorns::Template) -> usize {
                tpl.capacity_hint() #( + self.#fields.capacity_hint(tpl) )*
            }
            
            fn render_section<'section, 'content, E>(&'content self, mut section: ramhorns::Section<'section, 'content, E>, encoder: &mut E) -> Result<(), E::Error>
            where
        	'content: 'section,
                E: ramhorns::encoding::Encoder,
            {
                section.render_once(self, encoder)
            }

            fn render_field_escaped<E>(&self, hash: u64, _: &str, encoder: &mut E) -> Result<bool, E::Error>
            where
                E: ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_escaped )*
                    _ => Ok(false)
                }
            }

            fn render_field_unescaped<E>(&self, hash: u64, _: &str, encoder: &mut E) -> Result<bool, E::Error>
            where
                E: ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_unescaped )*
                    _ => Ok(false)
                }
            }

            fn render_field_section<'section, 'content, E>(&'content self, hash: u64, _: &str, section: ramhorns::Section<'section, 'content, E>, encoder: &mut E) -> Result<bool, E::Error>
            where
        	'content: 'section,
                E: ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_section )*
                    _ => Ok(false)
                }
            }

            fn render_field_inverse<'section, 'content, E>(&'content self, hash: u64, _: &str, section: ramhorns::Section<'section, 'content, E>, encoder: &mut E) -> Result<bool, E::Error>
            where
            	'content: 'section,
                E: ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_inverse )*
                    _ => Ok(false)
                }
            }
        }
    };

    // panic!("{}", tokens);

    TokenStream::from(tokens)
}
