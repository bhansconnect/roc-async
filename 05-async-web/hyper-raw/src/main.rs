use std::convert::Infallible;

use askama::Template;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

struct Link<'a> {
    name: &'a str,
    url: &'a str,
    description: &'a str,
}

// This template is ingested during build and should be extremely fast.
#[derive(Template)]
#[template(path = "roc-home.html")]
struct RocHomeTemplate<'a> {
    links: Vec<Link<'a>>,
}

#[inline(never)]
fn links() -> Vec<Link<'static>> {
    vec![
        Link {
            name: "Roc at Handmade Seattle",
            url: "https://media.handmade-seattle.com/roc-lang",
            description: "November 12, 2021 (very low-level explanation of how Roc's compiler makes programs run fast)",
        },
        Link {
            name: "Outperforming Imperative with Pure Functional Languages",
            url: "https://youtu.be/vzfy4EKwG_Y",
            description: "October 1, 2021 (about Roc's runtime performance and optimizer)",
        },
        Link {
            name: "A taste of Roc",
            url: "https://youtu.be/6qzWm_eoUXM",
            description: "September 23, 2021 (syntax, application examples)",
        },
        Link {
            name: "Roc at the Philly ETE conference",
            url: "https://youtu.be/cpQwtwVKAfU?t=75",
            description: "May 6, 2021 (platforms and applications)",
        },
        Link {
            name: "Roc on Zig Showtime",
            url: "https://youtu.be/FMyyYdFSOHA",
            description: "April 24, 2021 (making a platform)",
        },
        Link {
            name: "Roc at the Berlin FP Meetup",
            url: "https://youtu.be/ZnYa99QoznE?t=4790",
            description: "September 1, 2020 (overall vision for the language)",
        },
    ]
}

// This is intentionally a bad recursive fib to eat of compute time.
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[inline(never)]
async fn fake_db_call(delay_ms: u64) -> u64 {
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    // This is our dummy db call result.
    1
}

async fn root(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    // Path will aways start with a "/".
    // So skip the first result of "".
    let path = req.uri().path();
    let mut path = path.split("/").skip(1);
    let base = path.next();
    match (req.method(), base) {
        (&Method::GET, Some("")) => {
            // Root path just print hello world.
            *response.body_mut() = Body::from("Hello, World!");
        }
        (&Method::GET, Some("hello")) => {
            // Hello, but with a name.
            let first = path.next();
            let last = path.next();
            let body = match (first, last) {
                (Some(first), Some(last)) if !first.is_empty() && !last.is_empty() => {
                    format!("Hello, {} {}!", first, last)
                }
                (Some(first), _) if !first.is_empty() => format!("Hello, {}!", first),
                _ => "Hello, Mr. Nobody?".to_string(),
            };
            *response.body_mut() = Body::from(body);
        }
        (&Method::GET, Some("sleep")) => {
            // Sleep for X milliseconds Y times to simulate async calls.
            let delay_ms = path.next().map(|x| x.parse::<u64>());
            let reps = path.next().map(|x| x.parse::<u64>());
            match (delay_ms, reps) {
                (Some(Ok(delay_ms)), Some(Ok(reps))) if reps != 1 => {
                    let mut x = 0;
                    for _ in 0..reps {
                        x += fake_db_call(delay_ms).await;
                    }
                    *response.body_mut() = Body::from(format!("{} Naps Completed", x));
                }
                (Some(Ok(delay_ms)), None | Some(Ok(1))) => {
                    let x = fake_db_call(delay_ms).await;
                    *response.body_mut() = Body::from(format!("{} Nap Completed", x));
                }
                _ => {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                }
            }
        }
        (&Method::GET, Some("compute")) => {
            // Compute the nth fibonacci number.
            let n = path.next();
            if let Some(Ok(n)) = n.map(|x| x.parse::<u64>()) {
                *response.body_mut() = Body::from(fibonacci(n).to_string());
            } else {
                *response.status_mut() = StatusCode::BAD_REQUEST;
            }
        }
        (&Method::POST, Some("dup")) => {
            // Duplicate the body N times.
            let n = path.next().map(|x| x.parse::<usize>());
            let body = hyper::body::to_bytes(req.into_body()).await;
            match (n, body) {
                (Some(Ok(n)), Ok(body)) => {
                    *response.body_mut() = Body::from(body.repeat(n));
                }
                _ => {
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                }
            }
        }
        (&Method::GET, Some("template")) => {
            let template = RocHomeTemplate { links: links() };
            match template.render() {
                Ok(body) => {
                    *response.body_mut() = Body::from(body);
                }
                _ => {
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        }
        // TODO: add json encode and decode route
        // TODO: add DB access route (probably still mock with sleep but generate query or results)
        _ => *response.status_mut() = StatusCode::NOT_FOUND,
    }

    Ok(response)
}

#[tokio::main]
pub async fn main() {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(root)) }
    });

    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
