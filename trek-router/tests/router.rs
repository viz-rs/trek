use futures::{executor::block_on, future::ready, future::Future, stream::TryStreamExt};
use http::Method;
use hyper::Body;
use std::sync::Arc;
use trek_core::context::Context;
use trek_router::router::Router;

#[test]
fn new_router() {
    struct State {}
    let mut router = Router::<Context<State>>::new();

    router
        .get("/", |_| async { "home" })
        // scope v1
        .scope("/v1", |v1| {
            v1.get("/login", |_| async { "v1 login" })
                .post("/submit", |_| async { "1" })
                .delete("/read", |_| async { "2" });

            async fn get_users(_: Context<State>) -> String {
                String::from("get users")
            }

            fn get_user(_: Context<State>) -> impl Future<Output = String> {
                ready(String::from("get users :id"))
            }

            fn update_user(_: Context<State>) -> impl Future<Output = String> {
                async { String::from("update users :id") }
            }

            v1.scope("/users", |users| {
                users.get("", get_users);
                users.get("/:id", get_user);
                users.post("/:id", update_user);
            });
        })
        // scope v2
        .scope("/v2", |v2| {
            v2.get("/login", |_| async { "0" })
                .post("/submit", |_| async { "1" })
                .delete("/read", |_| async { "2" });
        })
        .get("/foo", |_| async { "3" })
        .post("/bar", |_| async { "4" })
        .delete("/baz", |_| async { "5" })
        // scope admin
        .scope("admin", |a| {
            a.any("", |_| async { "6" });
        });

    dbg!(&router);

    let r = router.find(&Method::GET, "/");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("GET")
                .uri("https://crates.io/")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "home",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::GET, "/v1/login");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("GET")
                .uri("https://crates.io/v1/login")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "v1 login",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::POST, "/v2/submit");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("POST")
                .uri("https://crates.io/v2/submit")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "1",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::GET, "/foo");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("GET")
                .uri("https://crates.io/foo")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "3",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::POST, "/bar");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("POST")
                .uri("https://crates.io/bar")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "4",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::DELETE, "/baz");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("DELETE")
                .uri("https://crates.io/bar")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "5",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::HEAD, "/admin");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("HEAD")
                .uri("https://crates.io/admin")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "6",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::OPTIONS, "/admin");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("OPTIONS")
                .uri("https://crates.io/admin")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "6",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::GET, "/v1/users");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, []);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("GET")
                .uri("https://crates.io/users")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "get users",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::GET, "/v1/users/fundon");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, [("id", "fundon")]);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("GET")
                .uri("https://crates.io/users/fundon")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "get users :id",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let r = router.find(&Method::POST, "/v1/users/fundon");
    assert!(r.is_some());
    let (h, p) = r.unwrap();
    assert_eq!(p, [("id", "fundon")]);
    block_on(async move {
        let cx = Context::new(
            Arc::new(State {}),
            hyper::Request::builder()
                .method("POST")
                .uri("https://crates.io/users/fundon")
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
            vec![],
        );
        let mut res = h.call(cx).await;
        assert_eq!(
            "update users :id",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });
}
