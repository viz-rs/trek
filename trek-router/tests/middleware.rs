use futures::{
    executor::block_on,
    future::{ready, BoxFuture, Future},
    stream::TryStreamExt,
};
use hyper::{Body, Method};
use std::sync::Arc;
use trek_core::{
    context::Context, handler::into_box_dyn_handler, handler::into_middleware,
    middleware::Middleware, parameters::Parameters, response::Response,
};
use trek_router::{
    resources::{Resource, Resources},
    router::Router,
};

#[test]
fn new_middleware() {
    struct State {}
    let mut router = Router::<Context<State>>::new();

    struct Middleware_A {}

    impl Middleware<Context<State>> for Middleware_A {
        fn call<'a>(&self, cx: Context<State>) -> BoxFuture<'a, Response> {
            Box::pin(async move {
                println!("middleware: {}", "A in");
                let res = cx.next().await;
                println!("middleware: {}", "A out");
                res
            })
        }
    }

    struct Middleware_B {}

    impl Middleware<Context<State>> for Middleware_B {
        fn call<'a>(&self, cx: Context<State>) -> BoxFuture<'a, Response> {
            Box::pin(async move {
                println!("middleware: {}", "B in");
                let res = cx.next().await;
                println!("middleware: {}", "B out");
                res
            })
        }
    }

    struct Middleware_C {}

    impl Middleware<Context<State>> for Middleware_C {
        fn call<'a>(&self, cx: Context<State>) -> BoxFuture<'a, Response> {
            Box::pin(async move {
                println!("middleware: {}", "C in");
                let res = cx.next().await;
                println!("middleware: {}", "C out");
                res
            })
        }
    }

    router.middleware(Middleware_A {});

    router.get("/", |_| async { "home" }).scope("/users", |r| {
        r.middleware(Middleware_B {}).get("", |_| async { "users" });
        r.clear_middleware()
            .middleware(Middleware_C {})
            .get("/", |_| async { "users index" })
            .get("/:name", |cx: Context<State>| {
                async move { cx.params::<String>().unwrap() }
            });
    });

    let router = Arc::new(router);

    let current_router = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/")
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
    });

    let current_router = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/users")
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
            "users",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let current_router = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/users/")
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
            "users index",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let current_router = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/users/crab")
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let route = current_router.find(method, &path);
        assert!(route.is_some());
        let (m, p) = route.unwrap();
        assert_eq!(p, [("name", "crab")]);
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
            "crab",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });
}
