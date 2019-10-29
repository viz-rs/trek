use futures::{
    executor::block_on,
    future::{ready, BoxFuture, Future},
    stream::TryStreamExt,
};
use hyper::{Body, Method};
use std::sync::Arc;
use trek_cookies::{ContextExt, Cookie, CookiesMiddleware};
use trek_core::{
    context::Context, handler::into_box_dyn_handler, handler::into_middleware,
    middleware::Middleware, parameters::Parameters, response::Response,
};
use trek_router::{
    resources::{Resource, Resources},
    router::Router,
};

#[test]
fn new_cookies() {
    struct State {}
    let mut router = Router::<Context<State>>::new();

    router.middleware(CookiesMiddleware {});

    router.get("/", |mut cx: Context<State>| {
        async move {
            cx.set_cookie(Cookie::new("logged_in", "no"));
            "home"
        }
    });

    let router = Arc::new(router);

    let current_router = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/")
            .header(
                hyper::header::COOKIE,
                hyper::header::HeaderValue::from_str("logged_in=yes; tz=Asia%2FShanghai").unwrap(),
            )
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let route = current_router.find(method, &path);
        assert!(route.is_some());
        let (m, p) = route.unwrap();
        assert_eq!(p, []);
        let cx = Context::new(
            Arc::new(State {}),
            req,
            p.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            m.to_vec(),
        );
        let mut res = cx.next().await;
        assert_eq!(
            "home",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
        assert_eq!(
            "logged_in=no",
            String::from_utf8(res.headers().get("set-cookie").unwrap().as_bytes().to_vec())
                .unwrap()
        )
    });
}
