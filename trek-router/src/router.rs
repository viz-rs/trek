use http::Method;
use path_tree::PathTree;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use trek_core::{
    handler::{into_box_dyn_handler, DynHandler, Handler},
    middleware::Middleware,
};

pub type Trees<Handler> = HashMap<Method, PathTree<Handler>>;

pub struct Router<Context> {
    path: String,
    trees: Trees<Box<DynHandler<Context>>>,
    middleware: Vec<Arc<dyn Middleware<Context>>>,
}

impl<Context> Router<Context> {
    pub fn new() -> Self {
        Self {
            path: "/".to_owned(),
            trees: Trees::new(),
            middleware: Vec::new(),
        }
    }

    pub fn middleware(&mut self, m: impl Middleware<Context>) -> &mut Self {
        self.middleware.push(Arc::new(m));
        self
    }

    pub fn scope<F>(&mut self, path: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let mut router = Router {
            path: Self::join_paths(&self.path, path),
            trees: self.trees.clone(),
            middleware: self.middleware.clone(),
        };

        f(&mut router);

        self.trees.clone_from(&router.trees);

        self
    }

    fn _handle(
        &mut self,
        method: Method,
        path: &str,
        handler: Box<DynHandler<Context>>,
    ) -> &mut Self {
        let path = &Self::join_paths(&self.path, path);
        self.trees
            .entry(method)
            .or_insert_with(PathTree::new)
            .insert(path, handler);
        self
    }

    pub fn handle(
        &mut self,
        method: Method,
        path: &str,
        handler: impl Handler<Context> + Clone,
    ) -> &mut Self {
        self._handle(method, path, into_box_dyn_handler(handler))
    }

    pub fn get(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::GET, path, handler)
    }

    pub fn post(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::POST, path, handler)
    }

    pub fn delete(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::DELETE, path, handler)
    }

    pub fn patch(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::PATCH, path, handler)
    }

    pub fn put(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::PUT, path, handler)
    }

    pub fn options(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::OPTIONS, path, handler)
    }

    pub fn head(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::HEAD, path, handler)
    }

    pub fn connect(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::CONNECT, path, handler)
    }

    pub fn trace(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::TRACE, path, handler)
    }

    pub fn any(&mut self, path: &str, handler: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(Method::GET, path, handler.clone())
            .handle(Method::POST, path, handler.clone())
            .handle(Method::DELETE, path, handler.clone())
            .handle(Method::PATCH, path, handler.clone())
            .handle(Method::PUT, path, handler.clone())
            .handle(Method::OPTIONS, path, handler.clone())
            .handle(Method::HEAD, path, handler.clone())
            .handle(Method::CONNECT, path, handler.clone())
            .handle(Method::TRACE, path, handler)
    }

    pub(crate) fn join_paths(a: &str, b: &str) -> String {
        if b.is_empty() {
            return a.to_owned();
        }
        a.trim_end_matches('/').to_owned() + "/" + b.trim_start_matches('/')
    }

    pub fn find<'a>(
        &'a self,
        method: &'a Method,
        path: &'a str,
    ) -> Option<(&'a Box<DynHandler<Context>>, Vec<(&'a str, &'a str)>)> {
        let tree = self.trees.get(method)?;
        tree.find(path)
    }
}

impl<Context> fmt::Debug for Router<Context> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Router")
            .field("path", &self.path)
            .finish()
    }
}
