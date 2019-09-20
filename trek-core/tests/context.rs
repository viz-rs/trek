use futures::{executor::block_on, stream};
use http::header::HeaderValue;
use hyper::{Body, Version};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use trek_core::context::Context;

#[test]
fn context() {
    #[derive(Debug)]
    struct State {}

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Query {
        q: String,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Json {
        name: String,
        age: u16,
    }

    let mut cx = Context::new(
        Arc::new(State {}),
        hyper::Request::builder()
            .uri("https://crates.io/search?q=web")
            .header("Content-Type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&Json {
                    name: "trek".to_owned(),
                    age: 1966,
                })
                .unwrap(),
            ))
            .unwrap(),
    );

    // dbg!(&cx);

    assert_eq!(cx.method(), "GET");
    assert_eq!(cx.version(), Version::HTTP_11);
    assert_eq!(cx.path(), "/search");

    assert_eq!(cx.header("Content-Type").unwrap(), "application/json");
    *cx.header_mut("Content-Type").unwrap() = HeaderValue::from_str("application/xml").unwrap();
    assert_eq!(cx.header("Content-Type").unwrap(), "application/xml");

    assert_eq!(cx.query_string(), "q=web");
    assert_eq!(
        cx.query::<Query>().unwrap(),
        Query {
            q: "web".to_owned()
        }
    );
    assert_eq!(
        block_on(cx.json::<Json>()).unwrap(),
        Json {
            name: "trek".to_owned(),
            age: 1966,
        }
    );

    let chunks: Vec<Result<_, ::std::io::Error>> = vec![Ok("hello"), Ok(" "), Ok("world")];
    let stream = stream::iter(chunks);
    let body = Body::wrap_stream(stream);

    let mut cx = Context::new(
        Arc::new(State {}),
        hyper::Request::builder()
            .method("POST")
            .uri("https://crates.io/")
            .body(body)
            .unwrap(),
    );

    // dbg!(&cx);

    assert_eq!(block_on(cx.string()).unwrap(), "hello world");
}
