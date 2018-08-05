#[macro_use] extern crate proc_macro_hack;
extern crate proc_macro2;

mod builtins;
mod parser;
mod state;

use parser::parse_body;
use proc_macro2::TokenStream;
use state::State;
use std::env;
use std::str::FromStr;

proc_macro_expr_impl! {
    pub fn router_dsl_impl(input: &str) -> String {
        let mut stream = TokenStream::from_str(input).unwrap().into_iter();
        let result = parse_body(&mut stream, &State::default());

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
