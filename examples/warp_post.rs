#[macro_use] extern crate serde_derive;
#[macro_use] extern crate warp;
#[macro_use] extern crate warp_dsl;
extern crate http;
extern crate pretty_env_logger;
extern crate serde;

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

    // GET /posts
    let get_posts = warp::get(warp::path("posts"))
        .map(|| {
            let posts = Posts {
                total: 5,
                posts: vec![make_post(1), make_post(20)],
            };

            warp::reply::json(&posts)
        });

    // POST /posts
    let post_posts = warp::post(
        warp::path("posts")
        .and(warp::body::json())
        .map(|mut post: Post| {
            post.id = Some(1);
            println!("Create post: {:?}", post);
            warp::reply::json(&post)
        }));

    // GET /post/:id
    let get_post = warp::get(
        path!("post" / usize).
        map(|id| warp::reply::json(&make_post(id))));

    let get_post = path!("post" / usize).
        and(warp_dsl::method(&http::Method::PATCH)).
        map(|id| format!("HEAD {}", id));

    

    let routes = get_posts.or(post_posts).or(get_post);
    let routes = get_post;

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3030));
}
