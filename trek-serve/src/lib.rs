use bytes::{BufMut, Bytes, BytesMut};
use futures::{
    future::{self, BoxFuture, Either},
    ready, stream, FutureExt, Stream, StreamExt,
};
// use headers::Header;
// use headers::HeaderMapExt;
// use headers::HeaderValue;
// use headers::IfModifiedSince;
// use headers::IfUnmodifiedSince;
// use headers::LastModified;
// use headers::Range;
// use http::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST, IF_MODIFIED_SINCE, IF_UNMODIFIED_SINCE};
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
// use http::Header;
use hyper::Response as HyperResponse;
use std::ffi::OsStr;
use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tokio::fs::{read, read_dir, File};
use tokio::io::AsyncRead;
use trek_core::{Body, Context, Handler, IntoResponse, Response, Result};

mod template;

use template::{render_breadcrumb, render_directory, render_file, DIRECTORY, FILE, FOLDER};

#[derive(Debug)]
pub struct ServeConfig {
    public: PathBuf,
    unlisted: Option<Vec<&'static str>>,
    // trailing_slash: Option<bool>,
}

impl ServeConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            public: path.into().canonicalize().unwrap(),
            unlisted: Some(vec![".DS_Store", ".git"]),
            // trailing_slash: None,
        }
    }

    pub fn unlisted(&mut self, list: Vec<&'static str>) -> &mut Self {
        if self.unlisted.is_some() {
            self.unlisted.as_mut().unwrap().append(&mut list.to_owned());
        } else {
            self.unlisted = Some(list);
        }
        self
    }
}

#[derive(Clone)]
pub struct ServeHandler {
    config: Arc<ServeConfig>,
}

fn file_stream(
    mut file: File,
    buf_size: usize,
    (start, end): (u64, u64),
) -> impl Stream<Item = std::result::Result<Bytes, io::Error>> + Send {
    let seek = async move {
        if start != 0 {
            file.seek(io::SeekFrom::Start(start)).await?;
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

                let mut chunk = buf.split_to(buf.len()).freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(chunk)))
            }))
        })
        .flatten()
}

async fn render(
    config: Arc<ServeConfig>,
    suffix_path: String,
    curr_path: &Path,
    path: PathBuf,
) -> Result<String> {
    let unlisted = config.unlisted.as_ref();
    let mut entries = read_dir(path.clone()).await?;
    let mut files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().into_string().unwrap();

        if unlisted.map_or_else(|| false, |list| list.contains(&file_name.as_ref())) {
            continue;
        }

        let file_type = if entry.file_type().await?.is_file() {
            FILE
        } else {
            FOLDER
        };

        let file_path = entry.path();
        let file_path = file_path.strip_prefix(path.clone()).unwrap();
        let file_ext = file_path
            .extension()
            .unwrap_or_else(|| OsStr::new(""))
            .to_owned()
            .into_string()
            .unwrap();

        files.push((
            curr_path
                .join(file_path)
                .into_os_string()
                .into_string()
                .unwrap(),
            file_name.to_owned(),
            file_type.to_owned(),
            file_ext,
            file_name,
        ));
    }

    files.sort_by_key(|f| f.1.to_owned());

    if !suffix_path.is_empty() {
        let parent = curr_path.parent().unwrap();
        files.insert(
            0,
            (
                parent.join("").into_os_string().into_string().unwrap(),
                parent
                    .file_name()
                    .unwrap()
                    .to_owned()
                    .into_string()
                    .unwrap(),
                DIRECTORY.to_owned(),
                "".to_owned(),
                "..".to_owned(),
            ),
        );
    }

    let mut breadcrumb: Vec<String> = curr_path
        .ancestors()
        .filter(|a| a.file_name().is_some())
        .map(|a| {
            render_breadcrumb(
                a.join("").to_str().unwrap(),
                a.file_name().unwrap().to_str().unwrap(),
            )
        })
        .collect();

    breadcrumb.reverse();

    let body = render_directory(
        curr_path.to_str().unwrap(),
        &breadcrumb.join(" "),
        &files
            .iter()
            .map(|f| {
                render_file(
                    f.0.to_owned(),
                    f.1.to_owned(),
                    f.2.to_owned(),
                    f.3.to_owned(),
                    f.4.to_owned(),
                )
            })
            .collect::<Vec<String>>()
            .join(""),
    );

    Ok(body)
}

impl ServeHandler {
    pub fn new(config: ServeConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    async fn send_file<State: Send + Sync + 'static>(
        config: Arc<ServeConfig>,
        cx: Context<State>,
    ) -> Result {
        let mut path = config.public.clone();
        let suffix_path = if cx.params.is_empty() {
            "".to_owned()
        } else {
            cx.params::<String>()
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "File not found"))?
        };

        path.push(suffix_path.clone());

        let file = File::open(path.clone()).await?;

        let metadata = file.metadata().await?;
        let file_type = metadata.file_type();

        let res = if file_type.is_file() {
            let content_type = match path.extension() {
                Some(ext) => mime_db::lookup(ext.to_owned().into_string().unwrap()),
                None => None,
            }
            .unwrap_or_else(|| "application/octet-stream");

            // TODO: https://github.com/hyperium/headers/pull/58
            // let last_modified = LastModified::from(metadata.modified()?);

            // let if_unmodified_since = cx.headers().get(IF_UNMODIFIED_SINCE);
            // if let Some(since) = if_unmodified_since {
            //     let since = IfUnmodifiedSince::from(since);
            //     let precondition = last_modified
            //         .map(|time| since.precondition_passes(time.into()))
            //         .unwrap_or(false);
            //     log::trace!(
            //         "if-unmodified-since? {:?} vs {:?} = {}",
            //         since,
            //         last_modified,
            //         precondition
            //     );
            //     if !precondition {
            //         let mut res = Response::new(Body::empty());
            //         *res.status_mut() = StatusCode::PRECONDITION_FAILED;
            //         return Cond::NoBody(res);
            //     }
            // }
            // let if_unmodified_since = cx.headers().get(IF_MODIFIED_SINCE);
            // let if_unmodified_since = cx.headers().get(CONTENT_TYPE);
            // let if_unmodified_since = cx.headers().get(HOST);
            // dbg!(if_unmodified_since);

            HyperResponse::builder()
                .header(CONTENT_TYPE, content_type)
                .header(CONTENT_LENGTH, metadata.len())
                .body(Body::wrap_stream(file_stream(file, 1, (0, 100))))
        } else if file_type.is_dir() {
            let index_file = path.join("index.html");

            let body = if index_file.exists() {
                String::from_utf8_lossy(&read(index_file).await?).to_string()
            } else {
                render(config, suffix_path, Path::new(cx.path()), path.clone()).await?
            };

            HyperResponse::builder()
                .header(CONTENT_TYPE, "text/html; charset=utf-8")
                .header(CONTENT_LENGTH, body.len())
                .body(Body::from(body))
        } else {
            HyperResponse::builder()
                .status(hyper::StatusCode::NOT_FOUND)
                .body(Body::empty())
        };

        Ok(res.map_err(|_| io::Error::new(io::ErrorKind::Other, "Can not response"))?)
    }
}

impl<State: Send + Sync + 'static> Handler<Context<State>> for ServeHandler {
    type Fut = BoxFuture<'static, Response>;

    fn call(&self, cx: Context<State>) -> Self::Fut {
        let fut = Self::send_file(self.config.clone(), cx);
        Box::pin(async move { fut.await.into_response() })
    }
}
