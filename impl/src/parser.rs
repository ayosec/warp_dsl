use proc_macro2::{Delimiter, TokenTree, TokenStream};
use std::iter::FromIterator;
use std::mem;

#[derive(Debug)]
pub(crate) enum DirectiveDecl {
    Complete,
    Filter(String),
}

#[derive(Debug)]
pub(crate) struct DirectiveTree {
    pub(crate) filters: Vec<String>,
    pub(crate) body: TokenStream,
    pub(crate) closure_args: Vec<String>,
    pub(crate) is_completed: bool,
}

impl DirectiveTree {
    pub(crate) fn from_tokens<S>(tokens: &mut S) -> DirectiveTree
    where
        S: Iterator<Item = TokenTree>
    {

        let mut filters = Vec::new();
        let mut is_completed = false;
        let mut current_filter = Vec::new();
        let mut closure_args = Vec::new();
        let body;

        'consumer: loop {
            let token = match tokens.next() {
                Some(t) => t,
                None => panic!("EOF looking for body directive"),
            };

            match &token {
                TokenTree::Ident(_) => {
                    current_filter.push(token.clone())
                }

                TokenTree::Group(group) => {
                    match group.delimiter() {
                        Delimiter::Parenthesis => {
                            current_filter.push(token.clone())
                        }

                        Delimiter::Brace => {
                            body = group.stream();
                            break 'consumer;
                        }

                        d => panic!("Expected one of '(', '{{'. Found '{:?}'", d),
                    }
                }

                TokenTree::Punct(punct) => {
                    match punct.as_char() {
                        '&' => {
                            let cf = mem::replace(&mut current_filter, Vec::new());
                            match parse_declaration(cf) {
                                DirectiveDecl::Filter(f) => filters.push(f),
                                DirectiveDecl::Complete => is_completed = true,
                            }
                        }

                        '|' => {
                            parse_closure_args(&mut closure_args, tokens);
                        }

                        c => panic!("Invalid character '{}' in directive declaration", c)
                    }
                }

                _ => panic!("Expected one of '(', '{{', or '&' in directive declaration"),
            }
        }

        let cf = mem::replace(&mut current_filter, Vec::new());
        match parse_declaration(cf) {
            DirectiveDecl::Filter(f) => filters.push(f),
            DirectiveDecl::Complete => is_completed = true,
        }

        DirectiveTree { filters, closure_args, body, is_completed }
    }
}

fn parse_declaration(mut tokens: Vec<TokenTree>) -> DirectiveDecl {

    let mut tokens = tokens.drain(..);

    match tokens.next() {
        None => {
            panic!("Found body without a directive")
        }

        Some(TokenTree::Ident(ident)) => {
            parse_declaration_by_name(ident.to_string().as_str(), &mut tokens)
        }

        Some(TokenTree::Group(group)) => {
            let expr = group.stream().to_string();
            DirectiveDecl::Filter(expr)
        }

        _ => unreachable!()
    }
}

fn parse_declaration_by_name(name: &str, tokens: &mut impl Iterator<Item = TokenTree>) -> DirectiveDecl {

    let args = match tokens.next() {
        Some(TokenTree::Group(group)) => Some(group.stream()),
        None => None,
        _ => panic!("Expected '(' after directive name"),
    };

    if tokens.next().is_some() {
        panic!("Too many tokens after directive declaration");
    }

    match name {
        "path" => {
            let args = args.expect("Missing arguments for 'path'");
            DirectiveDecl::Filter(format!("path!({})", args))
        }

        "connect" | "delete" | "get" | "head" | "options" | "patch" | "post" | "put" | "trace" => {
            if args.is_some() {
                panic!("'{}' directive does not take arguments", name);
            }

            let http_method = name.to_ascii_uppercase();
            DirectiveDecl::Filter(format!("::warp::is_method(&warp::http::Method::{})", http_method))
        }

        "complete" => {
            if args.is_some() {
                panic!("'complete' directive does not take arguments");
            }

            DirectiveDecl::Complete
        }

        _ => panic!("Invalid directive '{}'. Use (...) to use a custom filter.", name),
    }

}

fn parse_closure_args<S>(closure_args: &mut Vec<String>, tokens: &mut S)
where
    S: Iterator<Item = TokenTree>
{
    let mut args = Vec::new();
    while let Some(token) = tokens.next() {
        match &token {
            TokenTree::Punct(p) if p.as_char() == '|' => {
                let stream = TokenStream::from_iter(args.into_iter());
                closure_args.push(stream.to_string());
                return;
            }
            _ => { args.push(token.clone()); }
        }
    }

    panic!("EOF looking for '|'");
}
