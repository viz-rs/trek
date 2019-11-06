use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Error,
};
use std::sync::Arc;

use crate::{middleware::NotFound, Context, Router};

#[derive(Debug)]
pub struct Trek<State> {
    state: State,
    router: Router<Context<State>>,
}

impl<State: Default + Send + Sync + 'static> Trek<State> {
    pub fn with_state(state: State) -> Self {
        Self {
            state,
            router: Router::new(),
        }
    }

    pub fn router(&mut self) -> &mut Router<Context<State>> {
        &mut self.router
    }

    pub async fn run(self, addr: impl async_std::net::ToSocketAddrs) -> std::io::Result<()> {
        let addr = addr
            .to_socket_addrs()
            .await?
            .next()
            .ok_or(std::io::ErrorKind::InvalidInput)?;

        let builder = Server::try_bind(&addr).map_err(|e| {
            error!("error bind to {}: {}", addr, e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;

        info!("trek running on https://{}", addr);

        let state = Arc::new(State::default());
        let router = Arc::new(self.router);
        let not_found = Arc::new(NotFound::new());

        Ok(builder
            .serve(make_service_fn(move |_socket| {
                let state = state.clone();
                let router = router.clone();
                let not_found = not_found.clone();

                async move {
                    Ok::<_, Error>(service_fn(move |req| {
                        let state = state.clone();
                        let path = req.uri().path().to_owned();
                        let method = req.method().to_owned();
                        let middleware = router.middleware.clone();
                        let mut cx = Context::new(state, req, vec![], middleware.clone());

                        match router.find(&path, method) {
                            Some((m, p)) => {
                                cx.middleware.append(&mut m.clone());
                                cx.params = p
                                    .iter()
                                    .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                                    .collect();
                            }
                            None => {
                                cx.middleware.push(not_found.clone());
                            }
                        };

                        async move { Ok::<_, Error>(cx.next().await) }
                    }))
                }
            }))
            .await
            .map_err(|e| {
                error!("server error: {}", e);
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?)
    }
}

impl Trek<()> {
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl Default for Trek<()> {
    fn default() -> Self {
        Self::new()
    }
}
