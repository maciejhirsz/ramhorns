// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

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

	    	(name, token, hash, &field.ty)
	    })
	    .collect::<Vec<_>>();

	fields.sort_unstable_by(|a, b| (a.2).cmp(&b.2));

	let names = fields.iter().map(|(name, _, _, _)| name);

	let render_escaped = fields.iter().map(|(_, field, hash, _)| {
		quote! {
			#hash => ramhorns::escape(buf, self.#field),
		}
	});

	let render_unescaped = fields.iter().map(|(_, field, hash, _)| {
		quote! {
			#hash => buf.push_str(self.#field),
		}
	});

	let fields = fields.iter().map(|(_, field, _, _)| field);

	let tokens = quote! {
		impl#generics ramhorns::Context for #name#generics {
			const FIELDS: &'static [&'static str] = &[ #( #names ),* ];

			fn capacity_hint(&self, tpl: &ramhorns::Template) -> usize {
				tpl.capacity_hint() #( + self.#fields.len() )*
			}

			fn render_escaped(&self, hash: u64, buf: &mut String) {
				match hash {
					#( #render_escaped )*
					_ => {}
				}
			}

			fn render_unescaped(&self, hash: u64, buf: &mut String) {
				match hash {
					#( #render_unescaped )*
					_ => {}
				}
			}
		}
	};

	// panic!("{}", tokens);

    TokenStream::from(tokens).into()
}
