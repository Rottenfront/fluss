use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;
use crate::view_parser::ViewParser;

mod view_parser;
mod view_args_parser;

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ViewParser).into_token_stream().into()
}