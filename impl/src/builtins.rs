use parser::DeclItem;
use proc_macro2::{TokenStream, TokenTree};

#[derive(Debug)]
pub(crate) enum DirectiveDecl {
    Complete,
    HttpMethod(String),
    Filter(String),
}

pub(crate) fn parse(name: &str, tokens: &mut impl Iterator<Item = DeclItem>) -> DirectiveDecl {

    let args = match tokens.next() {
        Some(DeclItem::FnCall(stream)) => Some(stream),
        None => None,
        _ => panic!("Expected '(' after directive name"),
    };

    if tokens.next().is_some() {
        panic!("Too many tokens after directive declaration");
    }

    let expect_no_args = {
        let has_args = args.is_some();
        move || {
            if has_args {
                panic!("'{}' directive does not take arguments", name);
            }
        }
    };

    match name {
        "path" => {
            let args = args.expect("Missing arguments for 'path'");
            DirectiveDecl::Filter(format!("path!({})", args))
        }

        "connect" | "delete" | "get" | "head" | "options" | "patch" | "post" | "put" | "trace" => {
            expect_no_args();
            DirectiveDecl::HttpMethod(name.to_string())
        }

        "cookie" => {
            DirectiveDecl::Filter(cookie(args.expect("Missing arguments for 'cookie'")))
        }

        "complete" => {
            expect_no_args();
            DirectiveDecl::Complete
        }

        _ => panic!("Invalid directive '{}'. Use (...) to use a custom filter.", name),
    }

}

fn cookie(args: TokenStream) -> String {
    let args: Vec<_> = args.into_iter().collect();
    let func_name;
    let func_args;

    match args.len() {
        1 => {
            func_name = "::warp::filters::cookie::cookie";
            func_args = &args[0];
        }
        2 => {
            match &args[0] {
                TokenTree::Ident(i) if i.to_string() == "optional" => {
                    func_name = "::warp::filters::cookie::optional";
                    func_args = &args[1];
                }
                _ => panic!("Only 'optional' is a valid modifier for cookie. Found '{}'", args[0]),
            }
        }
        _ => panic!("Invalid arguments for 'cookie'")
    };

    format!("{}({})", func_name, func_args)
}
