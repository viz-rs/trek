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

    let cx = Context::new(
        Arc::new(State {}),
        hyper::Request::builder()
            .uri("https://crates.io/search?q=web")
            .body(hyper::Body::empty())
            .unwrap(),
    );

    assert_eq!(cx.path(), "/search");
    assert_eq!(cx.query_string(), "q=web");
    assert_eq!(
        cx.query::<Query>().unwrap(),
        Query {
            q: "web".to_owned()
        }
    );
}
