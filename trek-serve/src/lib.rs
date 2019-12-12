use futures::future::BoxFuture;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::Response as HyperResponse;
use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs::File;
use trek_core::{Body, Context, Handler, IntoResponse, Response, Result};

mod template;

mod utils;

use utils::*;

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
            let headers = cx.headers();

            Ok(file_respond(file, metadata, headers, content_type))
        } else if file_type.is_dir() {
            let index_file = path.join("index.html");

            if index_file.exists() {
                let file = File::open(index_file).await?;
                let metadata = file.metadata().await?;
                let headers = cx.headers();

                return Ok(file_respond(
                    file,
                    metadata,
                    headers,
                    "text/html; charset=utf-8",
                ));
                // String::from_utf8_lossy(&read(index_file).await?).to_string()
            }

            let body = render(config, suffix_path, Path::new(cx.path()), path.clone()).await?;

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
