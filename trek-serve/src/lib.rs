use bytes::{BufMut, Bytes, BytesMut};
use futures::{
    future::{self, BoxFuture, Either},
    ready, stream, FutureExt, Stream, StreamExt,
};
// use headers::LastModified;
use http::header::CONTENT_LENGTH;
use hyper::Response as HyperResponse;
use std::ffi::OsStr;
use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tokio::io::AsyncRead;
use trek_core::{Body, Context, Handler, IntoResponse, Response, Result};

const FILE: &'static str = "file";
const FOLDER: &'static str = "folder";
const DIRECTORY: &'static str = "directory";
const DIRECTORY_TEMPLATE: &'static str = include_str!("directory.html");
const BREADCRUMB_TEMPLATE: &'static str = r#"<a href="{{href}}">{{name}}/</a>"#;
const FILE_TEMPLATE: &'static str =
    r#"<li><a href="{{href}}" title="{{title}}" class="{{type}} {{ext}}">{{base}}</a></li>"#;

#[derive(Debug)]
pub struct ServeConfig {
    public: PathBuf,
}

impl ServeConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            public: path.into().canonicalize().unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct ServeHandler {
    config: Arc<ServeConfig>,
}

fn file_stream(
    mut file: tokio::fs::File,
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

impl ServeHandler {
    pub fn new(config: ServeConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    async fn send_file<State: Send + Sync + 'static>(
        mut path: PathBuf,
        cx: Context<State>,
    ) -> Result {
        let mut suffix_path = "".to_owned();

        if !cx.params.is_empty() {
            suffix_path = cx
                .params::<String>()
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "File not found"))?;
        }

        path.push(suffix_path.clone());

        let file = tokio::fs::File::open(path.clone()).await?;

        let metadata = file.metadata().await?;
        // let modified = LastModified::from(metadata.modified()?);
        let file_type = metadata.file_type();

        let res = if file_type.is_file() {
            // let ext = path.extension();

            HyperResponse::builder()
                .header(CONTENT_LENGTH, metadata.len())
                .body(Body::wrap_stream(file_stream(file, 1, (0, 100))))
        } else if file_type.is_dir() {
            let curr_path = Path::new(cx.path());
            let mut entries = tokio::fs::read_dir(path.clone()).await?;
            let mut files = Vec::new();

            if !suffix_path.is_empty() {
                let parent = curr_path.parent().unwrap();
                files.push(
                    FILE_TEMPLATE
                        .replace("{{href}}", parent.join("").to_str().unwrap())
                        .replace("{{title}}", parent.file_name().unwrap().to_str().unwrap())
                        .replace("{{type}}", DIRECTORY)
                        .replace("{{ext}}", "")
                        .replace("{{base}}", ".."),
                )
            }

            while let Some(entry) = entries.next_entry().await? {
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();
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
                    .to_str()
                    .unwrap();

                files.push(
                    FILE_TEMPLATE
                        .replace("{{href}}", curr_path.join(file_path).to_str().unwrap())
                        .replace("{{title}}", file_name)
                        .replace("{{type}}", file_type)
                        .replace("{{ext}}", file_ext)
                        .replace("{{base}}", file_name),
                );
            }

            let mut breadcrumb: Vec<String> = curr_path
                .ancestors()
                .filter(|a| a.file_name().is_some())
                .map(|a| {
                    BREADCRUMB_TEMPLATE
                        .replace("{{href}}", a.join("").to_str().unwrap())
                        .replace("{{name}}", a.file_name().unwrap().to_str().unwrap())
                })
                .collect();

            breadcrumb.reverse();

            let body = DIRECTORY_TEMPLATE
                .replace("{{title}}", curr_path.to_str().unwrap())
                .replace("{{breadcrumb}}", &breadcrumb.join(" "))
                .replace("{{files}}", &files.join(""));

            HyperResponse::builder()
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
        let config = self.config.clone();
        let fut = Self::send_file(config.public.clone(), cx);
        Box::pin(async move { fut.await.into_response() })
    }
}
