#[macro_use] extern crate warp;
#[macro_use] extern crate warp_dsl;

#[macro_use] extern crate serde_derive;
extern crate serde;

use std::str;
use warp::{body, Filter};

macro_rules! test_request {

    ($method:ident $path:expr; json($json:expr) => $routes:expr, status = $status:ident, body = $body:expr) => ({

        let response = warp::test::request()
            .method(stringify!($method))
            .path($path)
            .json(&$json)
            .reply(&$routes);

        assert_eq!(::warp::http::StatusCode::$status, response.status()); 

        let body = str::from_utf8(&response.body()[..]).unwrap();
        assert_eq!($body, body);
    });

    ($method:ident $path:expr => $routes:expr, status = $status:ident, body = $body:expr) => ({

        let response = warp::test::request()
            .method(stringify!($method))
            .path($path)
            .reply(&$routes);

        assert_eq!(::warp::http::StatusCode::$status, response.status()); 

        let body = str::from_utf8(&response.body()[..]).unwrap();
        assert_eq!($body, body);
    });
}

// Dummy resource to send and receive JSON objects

#[derive(Deserialize, Serialize)]
struct Resource {
    x: String,
    y: usize,
}

impl Resource {
    fn new<S: Into<String>>(x: S, y: usize) -> Resource {
        Resource { x: x.into(), y }
    }
}

#[test]
fn complex_router() {
    let routes = router!(

        // GET /resource/$id
        // Use '&' to combine directives

        get & path("resource" / usize) |id| {
            complete {
                warp::reply::json(&Resource::new(format!("0/{}", id), id))
            }
        }

        // Any DELETE request

        delete {
            complete {
                format!("DELETE request")
            }
        }

        // HTTP methods under path()

        path("resources") {

            get {

                path(u64) |id| {
                    complete {
                        format!("1: GET /resources/{}", id)
                    }
                }

                index {
                    complete {
                        format!("2: GET /resources")
                    }
                }

            }

            post {

                (body::json()) |r: Resource| {
                    complete {
                        format!("3: POST /resources y = {}", r.y)
                    }
                }

            }
        }

        // Cookies

        cookie(optional "test") |x: Option<String>| {
            path("cookie" / usize) |y| {
                complete {
                    let x = x.unwrap_or(String::new());
                    warp::reply::json(&Resource::new(x, y))
                }
            }
        }

    );

    test_request!(
        GET "/resource/1000" => routes,
        status = OK,
        body = r#"{"x":"0/1000","y":1000}"#);

    test_request!(
        GET "/resources/0001" => routes,
        status = OK,
        body = r#"1: GET /resources/1"#);

    test_request!(
        POST "/resources"; json(Resource::new("", 10)) => routes,
        status = OK,
        body = r#"3: POST /resources y = 10"#);

    test_request!(
        GET "/resources" => routes,
        status = OK,
        body = r#"2: GET /resources"#);

    test_request!(
        GET "/x" => routes,
        status = NOT_FOUND,
        body = r#""#);
}
