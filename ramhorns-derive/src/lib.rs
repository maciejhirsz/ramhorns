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

use fnv::FnvHasher;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, Error, Fields, ItemStruct, LitInt, Path};

use std::cmp::Ordering;
use std::hash::Hasher;

type UnitFields = Punctuated<syn::Field, Comma>;

struct Field {
    hash: u64,
    field: TokenStream2,
    callback: Option<Path>,
}

impl PartialEq for Field {
    fn eq(&self, other: &Field) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Field {}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Field) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Field) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

#[proc_macro_derive(Content, attributes(md, ramhorns))]
pub fn content_derive(input: TokenStream) -> TokenStream {
    let item: ItemStruct =
        syn::parse(input).expect("#[derive(Content)] can be only applied to structs");

    // panic!("{:#?}", item);

    let name = &item.ident;
    let generics = &item.generics;
    let type_params = item.generics.type_params();
    let unit_fields = UnitFields::new();

    let mut errors = Vec::new();

    let fields = match item.fields {
        Fields::Named(fields) => fields.named.into_iter(),
        Fields::Unnamed(fields) => fields.unnamed.into_iter(),
        _ => unit_fields.into_iter(),
    };

    let mut flatten = Vec::new();
    let md_callback: Path = syn::parse(quote!(::ramhorns::encoding::encode_cmark).into()).unwrap();
    let mut fields = fields
        .enumerate()
        .filter_map(|(index, field)| {
            let mut callback = None;
            let mut rename = None;
            let mut skip = false;

            let mut parse_attr = |attr: &Attribute| -> Result<(), Error> {
                use syn::punctuated::Pair;
                use syn::{spanned::Spanned, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

                let meta_list = match attr.parse_meta()? {
                    Meta::List(ml) => ml,
                    _ => {
                        return Err(Error::new(
                            attr.span(),
                            "missing attributes; did you mean `#[ramhorns(md)]`?",
                        ));
                    }
                };

                for nested_meta in meta_list.nested {
                    match nested_meta {
                        NestedMeta::Meta(Meta::Path(path)) if path.is_ident("skip") => {
                            skip = true;
                        }
                        NestedMeta::Meta(Meta::Path(path)) if path.is_ident("md") => {
                            callback = Some(md_callback.clone());
                        }
                        NestedMeta::Meta(Meta::Path(path)) if path.is_ident("flatten") => {
                            flatten.push(field.ident.as_ref().map_or_else(
                                || {
                                    let index = index.to_string();
                                    let lit = LitInt::new(&index, Span::call_site());
                                    quote!(#lit)
                                },
                                |ident| quote!(#ident),
                            ));
                            skip = true;
                        }
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(lit_str),
                            ..
                        })) if path.is_ident("rename") => rename = Some(lit_str.value()),
                        NestedMeta::Meta(Meta::List(MetaList {
                            path, mut nested, ..
                        })) if path.is_ident("callback") => {
                            if let Some(Pair::End(NestedMeta::Meta(Meta::Path(path)))) =
                                nested.pop()
                            {
                                callback = Some(path);
                            } else {
                                return Err(Error::new(
                                    nested.span(),
                                    "`callback` attribute in `ramhorns` takes one path identifier",
                                ));
                            }
                        }
                        _ => {
                            return Err(Error::new(
                                nested_meta.span(),
                                "not a valid attribute in `ramhorns`",
                            ));
                        }
                    }
                }
                Ok(())
            };

            errors.extend(
                field
                    .attrs
                    .iter()
                    .filter(|a| a.path.is_ident("ramhorns"))
                    .filter_map(|a| parse_attr(a).err()),
            );

            if skip {
                return None;
            }

            let (name, field) = field.ident.as_ref().map_or_else(
                || {
                    let index = index.to_string();
                    let lit = LitInt::new(&index, Span::call_site());
                    let name = rename.as_ref().cloned().unwrap_or(index);
                    (name, quote!(#lit))
                },
                |ident| {
                    let name = rename
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| ident.to_string());
                    (name, quote!(#ident))
                },
            );

            let mut hasher = FnvHasher::default();

            hasher.write(name.as_bytes());

            let hash = hasher.finish();

            Some(Field {
                hash,
                field,
                callback,
            })
        })
        .collect::<Vec<_>>();

    if !errors.is_empty() {
        let errors: Vec<_> = errors.into_iter().map(|e| e.to_compile_error()).collect();
        return quote! {
            fn _ramhorns_derive_compile_errors() {
                #(#errors)*
            }
        }
        .into();
    }

    fields.sort_unstable();

    let render_field_escaped = fields.iter().map(
        |Field {
             field,
             hash,
             callback,
             ..
         }| {
            if let Some(callback) = callback {
                quote! {
                    #hash => #callback(&self.#field, encoder).map(|_| true),
                }
            } else {
                quote! {
                    #hash => self.#field.render_escaped(encoder).map(|_| true),
                }
            }
        },
    );

    let render_field_unescaped = fields.iter().map(
        |Field {
             field,
             hash,
             callback,
             ..
         }| {
            if let Some(callback) = callback {
                quote! {
                    #hash => #callback(&self.#field, encoder).map(|_| true),
                }
            } else {
                quote! {
                    #hash => self.#field.render_unescaped(encoder).map(|_| true),
                }
            }
        },
    );

    let render_field_section = fields.iter().map(|Field { field, hash, .. }| {
        quote! {
            #hash => self.#field.render_section(section, encoder).map(|_| true),
        }
    });

    let render_field_inverse = fields.iter().map(|Field { field, hash, .. }| {
        quote! {
            #hash => self.#field.render_inverse(section, encoder).map(|_| true),
        }
    });

    let flatten = &*flatten;
    let fields = fields.iter().map(|Field { field, .. }| field);

    let where_clause = type_params
        .map(|param| quote!(#param: ::ramhorns::Content))
        .collect::<Vec<_>>();
    let where_clause = if !where_clause.is_empty() {
        quote!(where #(#where_clause),*)
    } else {
        quote!()
    };

    // FIXME: decouple lifetimes from actual generics with trait boundaries
    let tokens = quote! {
        impl#generics ::ramhorns::Content for #name#generics #where_clause {
            #[inline]
            fn capacity_hint(&self, tpl: &::ramhorns::Template) -> usize {
                tpl.capacity_hint() #( + self.#fields.capacity_hint(tpl) )*
            }

            #[inline]
            fn render_section<C, E>(&self, section: ::ramhorns::Section<C>, encoder: &mut E) -> std::result::Result<(), E::Error>
            where
                C: ::ramhorns::traits::ContentSequence,
                E: ::ramhorns::encoding::Encoder,
            {
                section.with(self).render(encoder)
            }

            #[inline]
            fn render_field_escaped<E>(&self, hash: u64, name: &str, encoder: &mut E) -> std::result::Result<bool, E::Error>
            where
                E: ::ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_escaped )*
                    _ => Ok(
                        #( self.#flatten.render_field_escaped(hash, name, encoder)? ||)*
                        false
                    )
                }
            }

            #[inline]
            fn render_field_unescaped<E>(&self, hash: u64, name: &str, encoder: &mut E) -> std::result::Result<bool, E::Error>
            where
                E: ::ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_unescaped )*
                    _ => Ok(
                        #( self.#flatten.render_field_unescaped(hash, name, encoder)? ||)*
                        false
                    )
                }
            }

            fn render_field_section<P, E>(&self, hash: u64, name: &str, section: ::ramhorns::Section<P>, encoder: &mut E) -> std::result::Result<bool, E::Error>
            where
                P: ::ramhorns::traits::ContentSequence,
                E: ::ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_section )*
                    _ => Ok(
                        #( self.#flatten.render_field_section(hash, name, section, encoder)? ||)*
                        false
                    )
                }
            }

            fn render_field_inverse<P, E>(&self, hash: u64, name: &str, section: ::ramhorns::Section<P>, encoder: &mut E) -> std::result::Result<bool, E::Error>
            where
                P: ::ramhorns::traits::ContentSequence,
                E: ::ramhorns::encoding::Encoder,
            {
                match hash {
                    #( #render_field_inverse )*
                    _ => Ok(
                        #( self.#flatten.render_field_inverse(hash, name, section, encoder)? ||)*
                        false
                    )
                }
            }
        }
    };

    // panic!("{}", tokens);

    TokenStream::from(tokens)
}
