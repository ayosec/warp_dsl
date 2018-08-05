use parser::DeclItem;

#[derive(Debug)]
pub(crate) enum DirectiveDecl {
    Complete,
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
            let http_method = name.to_ascii_uppercase();
            DirectiveDecl::Filter(format!("::warp::is_method(&warp::http::Method::{})", http_method))
        }

        "complete" => {
            expect_no_args();
            DirectiveDecl::Complete
        }

        _ => panic!("Invalid directive '{}'. Use (...) to use a custom filter.", name),
    }

}

