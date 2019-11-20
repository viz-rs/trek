#[macro_use]
extern crate log;

use bytes::{BufMut, BytesMut};
use futures::future::Either;
use std::pin::Pin;

use futures::{future, ready, stream, FutureExt, Stream, StreamExt};
use headers::LastModified;
use serde::{Deserialize, Serialize};
use std::task::Poll;
use tokio::io::AsyncRead;

use futures::future::BoxFuture;
use trek::middleware::Logger;
use trek::{
    into_box_dyn_handler, json, Context, ErrorResponse, Middleware, Resources, Response, Result,
    Trek,
};

fn file_stream(
    mut file: tokio::fs::File,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = std::result::Result<hyper::Chunk, std::io::Error>> + Send {
    use std::io::SeekFrom;

    let seek = async move {
        if start != 0 {
            file.seek(SeekFrom::Start(start)).await?;
        }
        Ok(file)
    };

    seek.into_stream()
        .map(move |result| {
            let mut buf = BytesMut::new();
            let mut len = end - start;
            let mut f = match result {
                Ok(f) => f,
                Err(f) => return Either::Left(stream::once(future::err(f))),
            };

            Either::Right(stream::poll_fn(move |cx| {
                if len == 0 {
                    return Poll::Ready(None);
                }
                if buf.remaining_mut() < buf_size {
                    buf.reserve(buf_size);
                }
                let n = match ready!(Pin::new(&mut f).poll_read_buf(cx, &mut buf)) {
                    Ok(n) => n as u64,
                    Err(err) => {
                        log::debug!("file read error: {}", err);
                        return Poll::Ready(Some(Err(err)));
                    }
                };

                if n == 0 {
                    log::debug!("file read found EOF before expected length");
                    return Poll::Ready(None);
                }

                let mut chunk = buf.take().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(hyper::Chunk::from(chunk))))
            }))
        })
        .flatten()
}

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

#[derive(Debug, Serialize)]
struct MyError {
    code: u16,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("MyError").finish()
    }
}

impl std::error::Error for MyError {}

impl ErrorResponse for MyError {
    fn error_response(&self) -> Response {
        let mut res = hyper::Response::new(hyper::Body::from("hello my error"));
        *res.status_mut() = hyper::StatusCode::from_u16(self.code).unwrap();
        res
    }
}

async fn send_file(cx: Context<()>) -> Result {
    let mut path = std::env::current_dir()?;
    path.push("examples/static");

    let suffix_path = cx
        .params::<String>()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"))?;

    path.push(suffix_path);

    dbg!(&path.extension());

    let file = tokio::fs::File::open(path).await?;
    // .await
    // .map_err(|_| MyError { code: 404 })?;

    dbg!(&file);

    let metadata = file.metadata().await?;
    let modified = metadata.modified().ok().map(LastModified::from).unwrap();

    dbg!(&metadata);
    dbg!(&metadata.len());
    dbg!(&metadata.file_type());
    dbg!(&modified);

    Ok(hyper::Response::builder()
        .body(hyper::Body::wrap_stream(file_stream(file, 1, (0, 100))))
        .unwrap())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    better_panic::install();

    let mut app = Trek::new();

    app.router()
        .middleware(Logger::new())
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
                    into_box_dyn_handler(|cx: Context<()>| {
                        async move { "user show: ".to_owned() + &cx.params::<String>().unwrap() }
                    }),
                ),
                (
                    Resources::Edit,
                    into_box_dyn_handler(|cx: Context<()>| {
                        async move { "user edit: ".to_owned() + &cx.params::<String>().unwrap() }
                    }),
                ),
            ],
        )
        .get("/users/:name/repos/:repo/issues/:id", |cx: Context<()>| {
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
        .get("/static/*", send_file);

    if let Err(e) = app.run("127.0.0.1:8000").await {
        error!("Error: {}", e);
    }
}
