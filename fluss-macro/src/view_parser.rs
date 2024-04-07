use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::Token;
use syn::token::Token;
use crate::view_args_parser::ViewArgsParser;

pub(crate) enum ViewParser {
    Func {
        name: syn::Ident,
        args: Option<ViewArgsParser>,
        childs: Vec<ViewParser>,
    },
    Expr(syn::Expr),
    Append(syn::Expr)
}

impl Parse for ViewParser {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.parse::<Token![|]>().is_ok() {
            return Ok(Self::Append(input.parse::<syn::Expr>().unwrap()))
        }

        if input.peek(syn::token::Brace) {
            if let Ok(expr) = input.parse::<syn::Expr>() {
                return Ok(Self::Expr(expr))
            }
        }

        if let Ok(ident) = input.parse::<syn::Ident>() {
            let mut args = None;

            if input.peek(syn::token::Paren) {
                let content: syn::parse::ParseBuffer;
                syn::parenthesized!(content in input);
                if let Ok(view_args) = content.parse::<ViewArgsParser>() {
                    args = Some(view_args);
                }
            }

            return if input.peek(syn::token::Brace) {
                let mut childs = Vec::new();

                let content: syn::parse::ParseBuffer;
                syn::braced!(content in input);

                while !content.is_empty() {
                    if let Ok(view_parser) = content.parse::<ViewParser>() {
                        childs.push(view_parser);
                    }

                    if content.peek(Token![,]) {
                        let _ = content.parse::<Token![,]>();
                    }
                }

                Ok(Self::Func {
                    name: ident,
                    args,
                    childs,
                })
            } else {
                Ok(Self::Func {
                    name: ident,
                    args,
                    childs: vec![],
                })
            }
        }

        Err(syn::Error::new(input.span(), "error when parse macros"))
    }
}

impl ToTokens for ViewParser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self {
            ViewParser::Func {
                name,
                args,
                childs
            } => {
                if childs.is_empty() {
                    tokens.extend(quote! {
                        {
                            let view = #name(#args);
                            view
                        }
                    })
                } else {
                    tokens.extend(quote! {
                        {
                            let mut ____childs: Vec<Box<dyn View>> = vec![];
                            let mut ____childs_extent: Vec<Box<dyn View>> = vec![];
                            let mut ____is_appened = false;
                            #(
                                let ____box__child: Box<dyn View> = Box::new(#childs);
                                ____childs.push(____box__child);
                                if ____is_appened {
                                    ____childs.append(&mut ____childs_extent);
                                    ____is_appened = false;
                                }
                            )*

                            let view = #name(#args ____childs);
                            view
                        }
                    })
                }
            }
            ViewParser::Expr(expr) => {
                tokens.extend(quote! {
                    {
                        #expr
                    }
                })
            }
            ViewParser::Append(expr) => {
                tokens.extend(quote! {
                    {
                        {
                            ____childs_extent.extend(#expr);
                            ____is_appened = true;
                        }
                    }
                })
            }
        }
    }
}
