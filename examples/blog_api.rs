#[macro_use] extern crate warp;
#[macro_use] extern crate warp_dsl;

extern crate serde;
#[macro_use] extern crate serde_derive;

#[derive(Debug,Serialize,Deserialize)]
struct Post {
    title: String,
    body: String,
}

use warp::{body, Filter};

fn main() {

    let routes = router!(
        path("blogs") {
            get & index {
                complete { "Get all blogs" }
            }

            path(u32) |blog_id| {
                get {
                    complete { format!("Details for blog {}", blog_id) }
                }

                delete {
                    complete { format!("Delete blog {}", blog_id) }
                }
            }
        }

        path("blog" / u32 / "posts") |blog_id| {
            post & index & (body::json()) |post: Post| {
                complete {
                    format!("Publish post '{}' with body '{}' in blog {}", post.title, post.body, blog_id)
                }
            }

            get & path(u32) & index |post_id| {
                complete {
                    format!("Read post {} in blog {}", post_id, blog_id)
                }
            }
        }
    );

    warp::serve(routes).run(([0; 4], 4000));
}
