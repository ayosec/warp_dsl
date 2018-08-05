#[macro_use] extern crate proc_macro_hack;
extern crate proc_macro2;

mod builder;
mod parser;

use builder::Builder;
use parser::DirectiveTree;
use proc_macro2::{TokenTree, TokenStream};
use std::env;
use std::mem;
use std::str::FromStr;

proc_macro_expr_impl! {
    pub fn router_dsl_impl(input: &str) -> String {
        let mut stream = TokenStream::from_str(input).unwrap().into_iter();
        let result = parse_body(&mut stream, Builder::default());

        if let Ok(debug) = env::var("WARPDSL_DEBUG") {
            if debug.contains("input") {
                eprintln!("======== [DEBUG:INPUT] =========");
                for token in TokenStream::from_str(input).unwrap() {
                    eprintln!("{:#?}", token);
                }
                eprintln!("================================");
            }

            if debug.contains("output") {
                eprintln!("======== [DEBUG:OUTPUT] ========");
                eprintln!("{}", result);
                eprintln!("================================");
            }
        }

        result
    }
}

fn parse_body<S>(stream: &mut S, builder_base: Builder) -> String
where
    S: Iterator<Item = TokenTree>
{
    let mut directives = Vec::new();
    let mut stream = stream.peekable();

    while stream.peek().is_some() {
        let mut builder = builder_base.clone();

        let directive = DirectiveTree::from_tokens(&mut stream);
        directive.closure_args.iter().for_each(|f| builder.append_closure_args(&f));
        directive.filters.iter().for_each(|f| builder.append_filter(&f));

        let result = {
            if directive.is_completed {
                if builder.filters.is_empty() {
                    builder.append_filter("any()");
                }

                builder.filters.push_str(".map(|");
                builder.filters.push_str(&builder.closure_args);
                builder.filters.push_str("| {");
                builder.filters.push_str(&directive.body.to_string());
                builder.filters.push_str("})");

                mem::replace(&mut builder, builder_base.clone()).filters
            } else {
                parse_body(&mut directive.body.into_iter(), builder)
            }
        };

        directives.push(result);
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
