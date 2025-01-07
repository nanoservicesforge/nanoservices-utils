use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::TokenTree;


#[proc_macro_attribute]
pub fn web_schema(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens for the struct definition
    let input_struct = parse_macro_input!(item as ItemStruct);
    let args: TokenStream2 = parse_macro_input!(args);

    let args_string = args.to_string();
    let has_serialize = args_string.contains("Serialize");
    let has_deserialize = args_string.contains("Deserialize");

    let mut traits: Vec<TokenStream2> = args
        .into_iter()
        .filter_map(|token| {
            if let TokenTree::Ident(ident) = token {
                Some(quote!(#ident))
            } else {
                None
            }
        })
        .collect();

    if !has_serialize {
        traits.push(quote!(serde::Serialize));
    }
    if !has_deserialize {
        traits.push(quote!(serde::Deserialize));
    }

    #[cfg(feature = "code-gen")]
    traits.push(quote!(schemars::JsonSchema));

    let prepared_traits = quote! {
        #(#traits),*
    };

    let expanded = quote! {
        #[derive(#prepared_traits)]
        #input_struct
    };

    // Return the generated tokens
    TokenStream::from(expanded)
}
