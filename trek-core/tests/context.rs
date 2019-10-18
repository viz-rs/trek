use futures::future::{ready, BoxFuture, Future};
use futures::{
    executor::block_on,
    stream::{self, TryStreamExt},
};
use http::header::HeaderValue;
use hyper::{Body, Version};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use trek_core::context::Context;
use trek_core::handler::*;
use trek_core::middleware::*;
use trek_core::response::*;

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

    struct M0 {}

    impl Middleware<Context<State>> for M0 {
        fn call(&self, cx: Context<State>) -> BoxFuture<'static, Response> {
            Box::pin(async move {
                let mut res = cx.next().await;

                let body =
                    String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap();
                *res.body_mut() = Body::from(body + " Trek!");

                res
            })
        }
    }

    async fn handler_async(_: Context<State>) -> String {
        String::from("Star")
    }

    fn handler(_: Context<State>) -> impl Future<Output = String> {
        // async { String::from("Star") }
        ready(String::from("Star"))
    }

    let mut cx = Context::new(
        Arc::new(State {}),
        hyper::Request::builder()
            .method("POST")
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
        // vec![],
        // vec![Arc::new(into_middleware(handler))],
        vec![Arc::new(M0 {}), Arc::new(into_middleware(handler_async))],
    );

    // // dbg!(&cx);

    assert_eq!(cx.method(), "POST");
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

    *cx.take_body() = body;

    // dbg!(&cx);

    assert_eq!(block_on(cx.string()).unwrap(), "hello world");

    *cx.header_mut("Content-Type").unwrap() =
        HeaderValue::from_str("application/x-www-form-urlencoded").unwrap();
    *cx.take_body() = Body::from("name=trek&age=1966");

    // dbg!(&cx);

    assert_eq!(
        block_on(cx.form::<Json>()).unwrap(),
        Json {
            name: "trek".to_owned(),
            age: 1966,
        }
    );

    let formdata: &[&[u8]] = &[
        b"--boundary\r",
        b"\n",
        b"Content-Disposition:",
        b" form-data; name=",
        b"\"foo\"",
        b"\r\n\r\n",
        b"field data",
        b"\r",
        b"\n--boundary--",
    ];

    let chunks: Vec<Result<_, ::std::io::Error>> = formdata.iter().map(|b| Ok(*b)).collect();
    let stream = stream::iter(chunks);
    let body = Body::wrap_stream(stream);

    *cx.header_mut("Content-Type").unwrap() =
        HeaderValue::from_str("multipart/form-data; boundary=boundary").unwrap();
    *cx.take_body() = body;

    // dbg!(&cx);

    block_on(async move {
        let mut multipart = cx.multipart().unwrap();

        while let Some(field) = multipart.next_field().await.unwrap() {
            // dbg!(&field);
            assert_eq!(field.headers.name, "foo");
            assert_eq!(field.data.read_to_string().await.unwrap(), "field data");
        }

        let mut res = cx.next().await;
        let body = String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap();
        assert_eq!(body, "Star Trek!");
    });
}
