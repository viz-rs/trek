use futures::{executor::block_on, future::ready, future::Future, stream::TryStreamExt};
use hyper::Body;
use std::sync::Arc;
use trek_core::context::Context;
use trek_core::handler::into_box_dyn_handler;
use trek_router::resources::{Resource, Resources};
use trek_router::router::Router;

#[test]
fn new_resources() {
    struct State {}
    let mut router = Router::<Context<State>>::new();

    struct Geocoder {}

    impl Geocoder {
        fn new(_: Context<State>) -> impl Future<Output = String> {
            ready(String::from("new: geocoder"))
        }

        fn show(_: Context<State>) -> impl Future<Output = String> {
            async { String::from("show: geocoder") }
        }
    }

    async fn book_new(_: Context<State>) -> String {
        String::from("new: book")
    }

    fn book_show(_: Context<State>) -> impl Future<Output = String> {
        async { String::from("show: book") }
    }

    router
        .get("/", |_| async { "home" })
        .resource(
            "/geocoder",
            &[
                (Resource::Show, into_box_dyn_handler(Geocoder::show)),
                (Resource::New, into_box_dyn_handler(Geocoder::new)),
            ],
        )
        .resources(
            "/books",
            &[
                (Resources::Show, into_box_dyn_handler(book_show)),
                (Resources::New, into_box_dyn_handler(book_new)),
            ],
        );

    let router = Arc::new(router);

    let mr = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/geocoder")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let r = mr.find(method, &path);
        assert!(r.is_some());
        let (m, p) = r.unwrap();
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
            "show: geocoder",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let mr = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/geocoder/new")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let r = mr.find(method, &path);
        assert!(r.is_some());
        let (m, p) = r.unwrap();
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
            "new: geocoder",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let mr = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/books/233")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let r = mr.find(method, &path);
        assert!(r.is_some());
        let (m, p) = r.unwrap();
        assert_eq!(p, [("book_id", "233")]);
        let cx = Context::new(
            Arc::new(State {}),
            req,
            p.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            m.to_vec(),
        );
        let s: u8 = cx.params().unwrap();
        assert_eq!(s, 233);
        let mut res = cx.next().await;
        assert_eq!(
            "show: book",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });

    let mr = router.clone();
    block_on(async move {
        let req = hyper::Request::builder()
            .method("GET")
            .uri("https://crates.io/books/new")
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();
        let method = req.method().to_owned();
        let path = req.uri().path().to_owned();
        let r = mr.find(method, &path);
        assert!(r.is_some());
        let (m, p) = r.unwrap();
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
            "new: book",
            String::from_utf8(res.body_mut().try_concat().await.unwrap().to_vec()).unwrap()
        );
    });
}
