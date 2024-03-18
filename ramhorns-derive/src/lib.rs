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

use bae::FromAttributes;
use fnv::FnvHasher;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Fields, ItemStruct, LitInt, LitStr, Path};

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

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

#[derive(FromAttributes)]
struct Ramhorns {
    skip: Option<()>,
    md: Option<()>,
    flatten: Option<()>,
    rename: Option<LitStr>,
    rename_all: Option<LitStr>,
    callback: Option<Path>,
}

enum RenameAll {
    UpperCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}
impl TryFrom<LitStr> for RenameAll {
    type Error = syn::Error;

    fn try_from(value: LitStr) -> Result<Self, Self::Error> {
        Ok(match value.value().as_str() {
            "UPPERCASE" => Self::UpperCase,
            "PascalCase" => Self::PascalCase,
            "camelCase" => Self::CamelCase,
            "sake_case" => Self::SnakeCase,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnakeCase,
            "kebab-case" => Self::KebabCase,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebabCase,
            _ => {
                return Err(syn::Error::new(value.span(), "Invalid `rename_all`"));
            }
        })
    }
}
impl RenameAll {
    fn rename(&self, input: &str) -> String {
        match &self {
            Self::UpperCase => heck::ToTitleCase::to_title_case(input),
            Self::PascalCase => heck::ToPascalCase::to_pascal_case(input),
            Self::CamelCase => heck::ToLowerCamelCase::to_lower_camel_case(input),
            Self::SnakeCase => heck::ToSnakeCase::to_snake_case(input),
            Self::ScreamingSnakeCase => heck::ToShoutySnakeCase::to_shouty_snake_case(input),
            Self::KebabCase => heck::ToKebabCase::to_kebab_case(input),
            Self::ScreamingKebabCase => heck::ToShoutyKebabCase::to_shouty_kebab_case(input),
        }
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

    let rename_all = match Ramhorns::try_from_attributes(&item.attrs) {
        Ok(Some(ramhorns)) => match ramhorns.rename_all {
            Some(lit_str) => match RenameAll::try_from(lit_str) {
                Ok(v) => Some(v),
                Err(error) => {
                    errors.push(error);
                    None
                }
            },
            None => None,
        },
        Ok(None) => None,
        Err(err) => {
            errors.push(err);
            None
        }
    };

    let mut flatten = Vec::new();
    let md_callback: Path = syn::parse(quote!(::ramhorns::encoding::encode_cmark).into()).unwrap();
    let mut fields = fields
        .enumerate()
        .filter_map(|(index, field)| {
            let mut callback = None;
            let mut rename = None;
            let mut skip = false;

            match Ramhorns::try_from_attributes(&field.attrs) {
                Ok(Some(ramhorns)) => {
                    if ramhorns.skip.is_some() {
                        skip = true;
                    }
                    if ramhorns.md.is_some() {
                        callback = Some(md_callback.clone());
                    }
                    if ramhorns.flatten.is_some() {
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
                    if let Some(lit_str) = ramhorns.rename {
                        rename = Some(lit_str.value());
                    }
                    if let Some(path) = ramhorns.callback {
                        callback = Some(path);
                    }
                }
                Ok(None) => (),
                Err(err) => errors.push(err),
            };

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
                    let name =
                        rename
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| match rename_all.as_ref() {
                                Some(rename) => rename.rename(ident.to_string().as_str()),
                                None => ident.to_string(),
                            });
                    (name, quote!(#ident))
                },
            );

            let mut hasher = FnvHasher::default();
            name.hash(&mut hasher);
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
