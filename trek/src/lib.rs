use hyper::{
    service::{make_service_fn, service_fn},
    Body, Error, Response, Server,
};
use std::sync::Arc;
use trek_core::context::Context;
use trek_router::router::Router;

pub struct App<State> {
    state: Arc<State>,
    router: Arc<Router<Context<State>>>,
}

impl App<()> {
    pub fn new() -> App<()> {
        Self {
            state: Arc::new(()),
            router: Arc::new(Router::new()),
        }
    }
}

impl Default for App<()> {
    fn default() -> App<()> {
        Self::new()
    }
}

impl<State: Send + Sync + 'static> App<State> {
    pub fn with_state(state: State) -> Self {
        Self {
            state: Arc::new(state),
            router: Arc::new(Router::new()),
        }
    }

    pub fn router(&mut self) -> &mut Router<Context<State>> {
        Arc::get_mut(&mut self.router).unwrap()
    }

    pub async fn run(self, addr: impl std::net::ToSocketAddrs) -> std::io::Result<()> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(std::io::ErrorKind::InvalidInput)?;

        let server = Server::bind(&addr).serve(make_service_fn(move |_| {
            let state = self.state.clone();
            let router = self.router.clone();
            async move {
                Ok::<_, Error>(service_fn(move |req| {
                    let path = req.uri().path().to_owned();
                    let method = req.method().to_owned();

                    let res = match router.find(method, &path) {
                        Some((m, p)) => {
                            let cx = Context::new(
                                state.clone(),
                                req,
                                p.iter()
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .collect(),
                                m.to_vec(),
                            );
                            cx.next()
                        }
                        None => Box::pin(async move {
                            Response::builder().status(404).body(Body::empty()).unwrap()
                        }),
                    };

                    async move { Ok::<_, Error>(res.await) }
                }))
            }
        }));

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }

        Ok(())
    }
}
