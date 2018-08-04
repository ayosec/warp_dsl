#[macro_use] extern crate proc_macro_hack;
//#[macro_use] extern crate quote;
//extern crate proc_macro;
extern crate proc_macro2;
//extern crate syn;

mod builder;

use builder::Builder;
use proc_macro2::{Delimiter, TokenTree, TokenStream};
use std::env;
use std::iter::FromIterator;
use std::str::FromStr;

proc_macro_expr_impl! {
    pub fn router_dsl_impl(input: &str) -> String {
        /////  DEBUG   ///////
        for token in TokenStream::from_str(input).unwrap() {
            println!("{:#?}", token);
        }
        /////  DEBUG   ///////
        let mut stream = TokenStream::from_str(input).unwrap().into_iter();
        let result = parse_body(&mut stream, Builder::default());

        if env::var("DSL_DEBUG").is_ok() {
            println!("[DEBUG:ROUTER]\n========\n{}\n========", result);
        }

        result
    }
}

fn parse_body<S>(stream: &mut S, mut builder: Builder) -> String
where
    S: Iterator<Item = TokenTree>
{
    let mut directives = Vec::new();

    while let Some(token) = stream.next() {
        let directive = match &token {
            TokenTree::Ident(ident) => {
                parse_directive(ident.to_string(), stream, builder.clone())
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                parse_directive_as_expr(group.stream(), stream, builder.clone())
            }
            TokenTree::Punct(punct) if punct.as_char() == '|' => {
                parse_closure_args(&mut builder, stream);
                continue;
            }
            _ => panic!("Invalid token at '{}'", token),
        };

        directives.push(directive);
    }

    let mut result = String::new();
    for directive in directives {
        if result.is_empty() {
            result.push('(');
        } else {
            result.push_str(".or(");
        }
        result.push_str(&directive);
        result.push(')');
    }

    result
}

enum Group {
    FnCall(TokenStream),
    Block(TokenStream),
}

fn parse_directive<S>(ident: String, tokens: &mut S, mut builder: Builder) -> String
where
    S: Iterator<Item = TokenTree>
{
    // Get the body of the directive, and its params

    let group = loop {
        if let Some(TokenTree::Group(group)) = tokens.next() {
            match group.delimiter() {
                Delimiter::Parenthesis => break Group::FnCall(group.stream()),
                Delimiter::Brace => break Group::Block(group.stream()),
                _ => ()
            };
        }

        panic!("Expected '(' or '{{' after '{}'", ident);
    };

    let fn_call;
    let body;

    match group {
        Group::FnCall(stream) => {
            fn_call = Some(stream);
            body = extract_body(&ident, tokens);
        }
        Group::Block(stream) => {
            fn_call = None;
            body = stream;
        }
    }

    match ident.as_str() {
        "path" => {
            let args = match fn_call {
                Some(x) => x,
                None => panic!("Missing arguments for 'path'"),
            };

            builder.append_filter(&format!("path!({})", args));
            parse_body(&mut body.into_iter(), builder.clone())
        }

        "connect" | "delete" | "get" | "head" | "options" | "patch" | "post" | "put" | "trace" => {
            if fn_call.is_some() {
                panic!("'{}' directive does not take arguments", ident);
            }

            builder.append_filter(&format!("::warp::is_method(&warp::http::Method::{})", ident.to_ascii_uppercase()));
            parse_body(&mut body.into_iter(), builder.clone())
        }

        "complete" => {
            if fn_call.is_some() {
                panic!("'complete' directive does not take arguments");
            }

            if builder.filters.is_empty() {
                builder.append_filter("any()");
            }

            builder.filters.push_str(".map(|");
            builder.filters.push_str(&builder.extactors);
            builder.filters.push_str("| {");
            builder.filters.push_str(&body.to_string());
            builder.filters.push_str("})");
            builder.filters
        }

        _ => panic!("Invalid directive '{}'", ident),
    }
}

fn parse_directive_as_expr<S>(expr: TokenStream, tokens: &mut S, mut builder: Builder) -> String
where
    S: Iterator<Item = TokenTree>
{
    let body = extract_body(&expr.to_string(), tokens);
    builder.append_filter(&expr.to_string());
    parse_body(&mut body.into_iter(), builder.clone())
}

fn extract_body<S>(directive: &str, tokens: &mut S) -> TokenStream
where
    S: Iterator<Item = TokenTree>
{
    if let Some(TokenTree::Group(group)) = tokens.next() {
        if group.delimiter() == Delimiter::Brace {
            return group.stream();
        }
    }

    panic!("Missing body for directive '{}'", directive);
}

fn parse_closure_args<S>(builder: &mut Builder, tokens: &mut S)
where
    S: Iterator<Item = TokenTree>
{
    let mut args = Vec::new();
    while let Some(token) = tokens.next() {
        match &token {
            TokenTree::Punct(p) if p.as_char() == '|' => {
                let stream = TokenStream::from_iter(args.into_iter());
                builder.append_closure_args(&stream.to_string());
                return;
            }
            _ => { args.push(token.clone()); }
        }
    }

    panic!("EOF looking for '|'");
}
