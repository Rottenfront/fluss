use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::Token;

pub(crate) struct ViewArgsParser {
    args: Vec<syn::Expr>
}

impl Parse for ViewArgsParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();

        while !input.is_empty() {
            if let Ok(expr) = input.parse::<syn::Expr>() {
                args.push(expr);
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>();
            }
        }

        Ok(Self {
            args,
        })
    }
}

impl ToTokens for ViewArgsParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let args = &self.args;

        if !args.is_empty() {
            tokens.extend(quote! {
                #(#args),*,
            })
        }
    }
}