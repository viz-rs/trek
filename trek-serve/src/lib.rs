use bytes::{BufMut, BytesMut};
use futures::{
    future::{self, BoxFuture, Either},
    ready, stream, FutureExt, Stream, StreamExt,
};
use headers::LastModified;
use std::{
    io,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
    task::Poll,
};
use tokio::io::AsyncRead;
use trek_core::{Body, Chunk, Context, Handler, IntoResponse, Response, Result};

#[derive(Debug)]
pub struct ServeConfig {
    public: PathBuf,
}

impl ServeConfig {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            public: path.into(),
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
) -> impl Stream<Item = std::result::Result<Chunk, io::Error>> + Send {
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

                let mut chunk = buf.take().freeze();
                if n > len {
                    chunk = chunk.split_to(len as usize);
                    len = 0;
                } else {
                    len -= n;
                }

                Poll::Ready(Some(Ok(Chunk::from(chunk))))
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
        let suffix_path = cx
            .params::<String>()
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "File not found"))?;

        path.push(suffix_path);

        dbg!(&path.extension());

        let file = tokio::fs::File::open(path).await?;

        dbg!(&file);

        let metadata = file.metadata().await?;
        let modified = metadata.modified().ok().map(LastModified::from).unwrap();

        dbg!(&metadata);
        dbg!(&metadata.len());
        dbg!(&metadata.file_type());
        dbg!(&modified);

        Ok(Response::new(Body::wrap_stream(file_stream(
            file,
            1,
            (0, 100),
        ))))
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
