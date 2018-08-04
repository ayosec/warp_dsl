#[macro_use] extern crate proc_macro_hack;

#[allow(unused_imports)]
#[macro_use]
extern crate warp_dsl_impl;

pub use warp_dsl_impl::*;

proc_macro_expr_decl! {
    router! => router_dsl_impl
}
