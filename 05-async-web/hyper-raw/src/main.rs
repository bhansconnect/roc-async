use std::convert::Infallible;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

// This is intentionally a bad recursive fib to eat of compute time.
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
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
            // Sleep for X milliseconds to simulate a async call.
            let time = path.next();
            if let Some(Ok(time)) = time.map(|x| x.parse::<u64>()) {
                tokio::time::sleep(tokio::time::Duration::from_millis(time)).await;
                *response.body_mut() = Body::from("Nap Completed");
            } else {
                *response.status_mut() = StatusCode::BAD_REQUEST;
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
        // TODO: add html template route
        // TODO: add json encode and decode route
        // TODO: add high mem/generation route (probably with dynamic list)
        // TODO: add DB access route (can start by mocking with sleeping)
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