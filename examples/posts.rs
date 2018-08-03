extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate pretty_env_logger;
extern crate warp;
#[macro_use] extern crate warp_dsl;

use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct Post {
    id: Option<usize>,
    title: String,
    body: String,
    public: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Posts {
    total: usize,
    posts: Vec<Post>,
}

fn make_post(id: usize) -> Post {
    Post {
        id: Some(id),
        title: format!("Post #{}", id),
        body: String::from("<b>...</b>"),
        public: id > 10,
    }
}

fn main() {
    pretty_env_logger::init();

    let routes = router! {

        // GET /posts
        // POST /posts

        path("posts") {
            get {
                complete {
                    let posts = Posts {
                        total: 5,
                        posts: vec![make_post(1), make_post(20)],
                    };
                    warp::reply::json(&posts)
                }
            }

            post {
                body::json() { |mut post: Post|
                    complete {
                        post.id = Some(1);
                        println!("Create post: {:?}", post);
                        warp::reply::json(&post)
                    }
                }
            }
        }

        // GET /post/:id
        get & path("post" / usize) { |id|
            complete {
                let post = make_post(id);
                warp::reply::json(&post)
            }
        }

    };

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030));
}
