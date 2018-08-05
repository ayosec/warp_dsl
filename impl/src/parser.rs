use builtins::{self, DirectiveDecl};
use proc_macro2::{Delimiter, TokenTree, TokenStream};
use state::State;
use std::iter::FromIterator;
use std::mem;

#[derive(Debug)]
pub(crate) struct DirectiveTree {
    pub(crate) filters: Vec<String>,
    pub(crate) http_methods: Vec<String>,
    pub(crate) body: TokenStream,
    pub(crate) closure_args: Vec<String>,
    pub(crate) is_completed: bool,
}

pub(crate) enum DeclItem {
    Ident(String),
    FnCall(TokenStream),
}

impl DirectiveTree {
    pub(crate) fn from_tokens<S>(tokens: &mut S) -> DirectiveTree
    where
        S: Iterator<Item = TokenTree>
    {

        let mut filters = Vec::new();
        let mut is_completed = false;
        let mut http_methods = Vec::new();
        let mut current_filter = Vec::new();
        let mut closure_args = Vec::new();
        let body;

        'consumer: loop {
            let token = tokens.next().expect("EOF looking for body directive");

            match &token {
                TokenTree::Ident(_) => {
                    current_filter.push(DeclItem::Ident(token.to_string()))
                }

                TokenTree::Group(group) => {
                    match group.delimiter() {
                        Delimiter::Parenthesis => {
                            current_filter.push(DeclItem::FnCall(group.stream()))
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
                                DirectiveDecl::HttpMethod(m) => http_methods.push(m),
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
            DirectiveDecl::HttpMethod(m) => http_methods.push(m),
            DirectiveDecl::Complete => is_completed = true,
        }

        DirectiveTree { filters, http_methods, closure_args, body, is_completed }
    }
}

pub(crate) fn parse_body<S>(stream: &mut S, state: &State) -> String
where
    S: Iterator<Item = TokenTree>
{
    let mut directives = Vec::new();
    let mut stream = stream.peekable();

    while stream.peek().is_some() {
        let mut state = state.clone();

        let directive = DirectiveTree::from_tokens(&mut stream);
        directive.closure_args.iter().for_each(|a| state.append_closure_args(&a));
        directive.filters.iter().for_each(|f| state.append_filter(&f));
        directive.http_methods.iter().for_each(|m| state.append_http_method(&m));

        let result = {
            if directive.is_completed {
                if state.filters.is_empty() {
                    state.append_filter("::warp::any()");
                }

                let mut filters = state.filters;
                filters.push_str(".map(|");
                filters.push_str(&state.closure_args);
                filters.push_str("| {");
                filters.push_str(&directive.body.to_string());
                filters.push_str("})");

                match state.http_method {
                    Some(m) => format!("::warp::{}({})", m, filters),
                    None => filters,
                }
            } else {
                parse_body(&mut directive.body.into_iter(), &state)
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

    if result.is_empty() {
        panic!("Missing routes");
    }

    result
}

fn parse_declaration(mut tokens: Vec<DeclItem>) -> DirectiveDecl {

    let mut tokens = tokens.drain(..);

    match tokens.next() {
        None => {
            panic!("Found body without a directive")
        }

        Some(DeclItem::Ident(ident)) => {
            builtins::parse(ident.as_str(), &mut tokens)
        }

        Some(DeclItem::FnCall(stream)) => {
            let expr = stream.to_string();
            DirectiveDecl::Filter(expr)
        }
    }
}

fn parse_closure_args<S>(closure_args: &mut Vec<String>, tokens: &mut S)
where
    S: Iterator<Item = TokenTree>
{
    let mut args = Vec::new();
    loop {
        let token = tokens.next().expect("EOF looking for '|'");

        match &token {
            TokenTree::Punct(p) if p.as_char() == '|' => break,
            _ => (),
        }

        args.push(token);
    }

    let stream = TokenStream::from_iter(args.into_iter());
    closure_args.push(stream.to_string());
}
