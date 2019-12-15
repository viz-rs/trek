#[macro_use]
extern crate log;

use http::Extensions;

use serde::{Deserialize, Serialize};

use futures::future::BoxFuture;
use trek::middleware::Logger;
use trek::middleware::{Cookie, CookiesContextExt, CookiesMiddleware};
use trek::{into_box_dyn_handler, json, Context, Middleware, Resources, Response, Trek};
use trek_serve::{ServeConfig, ServeHandler};

struct MiddlewareA {}
struct MiddlewareB {}
struct MiddlewareC {}
struct MiddlewareD {}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareA {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware A: {}", "In");
            let res = cx.next().await;
            info!("Middleware A: {}", "Out");
            res
        })
    }
}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareB {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware B: {}", "In");
            let res = cx.next().await;
            info!("Middleware B: {}", "Out");
            res
        })
    }
}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareC {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware C: {}", "In");
            let res = cx.next().await;
            info!("Middleware C: {}", "Out");
            res
        })
    }
}

impl<State: Sync + Send + 'static> Middleware<Context<State>> for MiddlewareD {
    fn call<'a>(&'a self, cx: Context<State>) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            info!("Middleware D: {}", "In");
            let res = cx.next().await;
            info!("Middleware D: {}", "Out");
            res
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    name: String,
    repo: String,
    id: u64,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    pretty_env_logger::init();
    better_panic::install();

    type ContextState = Context<Extensions>;
    let mut app = Trek::with_state(Extensions::new());

    app.router()
        .middleware(Logger::new())
        .middleware(CookiesMiddleware::new())
        .middleware(MiddlewareA {})
        .middleware(MiddlewareB {})
        .get("/", |_| async { "hello" })
        .get("/rust", |_| async { "rust" })
        .get("/2018", |_| async { "2018" })
        .resources(
            "/users",
            &[
                (
                    Resources::Show,
                    into_box_dyn_handler(|cx: ContextState| {
                        let param = cx.params::<String>().unwrap_or_else(|_| "".to_owned());
                        cx.state().get::<i32>();
                        cx.get::<&str>();
                        async move { "user show: ".to_owned() + &param }
                    }),
                ),
                (
                    Resources::Edit,
                    into_box_dyn_handler(|mut cx: ContextState| {
                        let v = cx
                            .get_cookie("name")
                            .and_then(|v| v.value().parse::<u64>().ok())
                            .unwrap_or_else(|| 0);
                        cx.set_cookie(Cookie::new("name", (v + 1).to_string()));
                        let param = cx.params::<String>().unwrap_or_else(|_| "".to_owned());
                        cx.state().get::<i32>();
                        cx.set::<i32>(233);
                        cx.set::<&str>("hello");
                        dbg!(cx.get::<i32>());
                        dbg!(cx.get::<&str>());
                        async move { "user edit: ".to_owned() + &param }
                    }),
                ),
            ],
        )
        .get("/users/:name/repos/:repo/issues/:id", |cx: ContextState| {
            async move { json(&cx.params::<UserInfo>().unwrap()) }
        })
        .scope("/admin", |a| {
            a.middleware(MiddlewareC {});
            a.get("", |_| async { "hello /admin" });
            a.scope("/", |b| {
                b.middleware(MiddlewareD {});
                b.get("", |_| async { "hello /admin/" });
                b.get("users", |_| async { "hello /admin/users" });
            });
        })
        .any("/anywhere", |_| async { "Anywhere" })
        .get(
            "/static/*",
            ServeHandler::new({
                let mut config = ServeConfig::new("static/");
                config.unlisted(vec![".gitignore"]);
                config
            }),
        );
    // .get("/static/*", ServeHandler::new(ServeConfig::new("..")));

    if let Err(e) = app.run("127.0.0.1:8000").await {
        error!("Error: {}", e);
    }

    Ok(())
}
