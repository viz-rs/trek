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

    pub fn unlisted(&mut self, mut list: Vec<&'static str>) -> &mut Self {
        if self.unlisted.is_some() {
            self.unlisted.as_mut().unwrap().append(&mut list);
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
        let mut suffix_path = String::new();

        if !cx.params.is_empty() {
            if let Ok(p) = cx.params::<String>() {
                suffix_path += &p;
            }
        };

        path.push(suffix_path.clone());

        let file = File::open(path.clone()).await?;
        let metadata = file.metadata().await?;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            return Ok(respond(
                file,
                metadata,
                cx.headers(),
                path.extension()
                    .and_then(|ext| {
                        ext.to_str()
                            .and_then(|ext| mime_db::lookup(ext))
                            .and_then(|ext| Some(ext.to_owned()))
                    })
                    .unwrap_or_else(|| mime::APPLICATION_OCTET_STREAM.to_string()),
            ));
        }

        let is_dir = file_type.is_dir();

        if is_dir {
            let index_file = path.join("index.html");

            if index_file.exists() {
                let file = File::open(index_file.clone()).await?;
                let metadata = file.metadata().await?;

                if metadata.file_type().is_file() {
                    return Ok(respond(
                        file,
                        metadata,
                        cx.headers(),
                        mime::TEXT_HTML_UTF_8.to_string(),
                    ));
                }
            }
        }

        let res = if is_dir {
            let body = render(config, path, Path::new(cx.path()), suffix_path).await?;

            HyperResponse::builder()
                .header(CONTENT_TYPE, mime::TEXT_HTML_UTF_8.to_string())
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
