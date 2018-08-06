# warp-dsl

[![Build Status](https://travis-ci.org/ayosec/warp_dsl.svg?branch=master)](https://travis-ci.org/ayosec/warp_dsl)

DSL to write routes for [warp](https://github.com/seanmonstar/warp), inspired by [Akka HTTP](https://doc.akka.io/docs/akka-http/current/routing-dsl/index.html).

At this moment, this implementation is just a proof of concept.

It works on Rust stable, using [proc-macro-hack](https://github.com/dtolnay/proc-macro-hack).

## Getting Started

1. Add the following dependencies to your `Cargo.toml`:

    ```toml
    [dependencies]
    warp = "0.1"
    warp_dsl = "0.0.1"
    ```

2. Use `#[macro_use]` to declare both crates:

    ```rust
    #[macro_use] extern crate warp;
    #[macro_use] extern crate warp_dsl;
    ```

3. Write some routes:

    ```rust
    use warp::Filter;

    fn main() {
        let routes = router!(
            path("foo" / u64) |num| {
                get {
                    complete { format!("GET /foo/{}\n", num) }
                }

                post {
                    complete { format!("POST /foo/{}\n", num) }
                }
            }
        );

        warp::serve(routes).run(([0; 4], 3030));
    }
    ```

Then, type `cargo run` to start the HTTP server with these routes.

```bash
$ cargo run &

$ curl -X POST localhost:3030/foo/1 
POST /foo/1
```

## Writing Routes

**This section is just a draft. It needs to be expanded.**

This crate uses the concept of [directives](https://doc.akka.io/docs/akka-http/current/routing-dsl/directives/index.html) from Akka HTTP.

Some common [filters](https://docs.rs/warp/0.1.0/warp/filters/index.html) are recognized by the parser:

| Directive               | Expression                          |
|-------------------------|-------------------------------------|
| `path(...)`             | `path!(...)`                        |
| `index`                 | `warp::index()`                     |
| `get`, `post`, `put`, … | HTTP methods                        |
| `cookie("..")`          | `::warp::filters::cookie::cookie`   |
| `cookie(optional "..")` | `::warp::filters::cookie::optional` |

Any function can be used as a directive if:

* It returns a [`Filter`](https://docs.rs/warp/0.1.0/warp/trait.Filter.html).
* The expression is surrounded by parenthesis.

Nested directives are combined using [`Filter::and`](https://docs.rs/warp/0.1.0/warp/trait.Filter.html#method.and). Directives in the same level are combined using [`Filter::or`](https://docs.rs/warp/0.1.0/warp/trait.Filter.html#method.or).

The `&` operator can be used to combine nested directives. `foo & bar { ... }` is equivalent to `foo { bar { ... } }`.

HTTP methods are moved to the top of the filter, since in wrap 0.1 they can't be used with `and()`.

The response is set using the `complete` directive.

### Debugging

If you have problems writing the routes, it may help to verify the generated code. Set the `WARPDSL_DEBUG` to `output`, and the code for every call to `router!()` is printed to stderr.

With the code of the *Getting Started* section, you get the following output:

    ======== [DEBUG:OUTPUT] ========
    (((::warp::get((path!("foo" / u64)).map(|num| {format ! ( "GET /foo/{}\n" , num )}))))
    .or((::warp::post((path!("foo" / u64)).map(|num| {format ! ( "POST /foo/{}\n" , num )})))))
    ================================
