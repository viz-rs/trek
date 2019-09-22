use http::Method;
use path_tree::PathTree;
use std::collections::HashMap;

pub type Trees<H> = HashMap<Method, PathTree<H>>;

#[derive(Clone, Debug)]
pub struct Router<H> {
    path: String,
    trees: Trees<H>,
    middleware: Vec<H>,
}

impl<H: Clone> Router<H> {
    pub fn new() -> Self {
        Self {
            path: "/".to_owned(),
            trees: Trees::new(),
            middleware: Vec::new(),
        }
    }

    pub fn middleware(&mut self, handler: H) -> &mut Self {
        self.middleware.push(handler);
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

        self
    }

    fn _handle(&mut self, method: Method, path: &str, handler: H) -> &mut Self {
        self.trees
            .entry(method)
            .or_insert_with(PathTree::new)
            .insert(path, handler);
        self
    }

    pub fn handle(&mut self, method: Method, path: &str, handler: H) -> &mut Self {
        self._handle(method, path, handler)
    }

    pub fn get(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::GET, path, handler)
    }

    pub fn post(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::POST, path, handler)
    }

    pub fn delete(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::DELETE, path, handler)
    }

    pub fn patch(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::PATCH, path, handler)
    }

    pub fn put(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::PUT, path, handler)
    }

    pub fn options(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::OPTIONS, path, handler)
    }

    pub fn head(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::HEAD, path, handler)
    }

    pub fn connect(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::CONNECT, path, handler)
    }

    pub fn trace(&mut self, path: &str, handler: H) -> &mut Self {
        self.handle(Method::TRACE, path, handler)
    }

    pub fn any(&mut self, path: &str, handler: H) -> &mut Self {
        let path = &Self::join_paths(&self.path, path);
        self._handle(Method::GET, path, handler.to_owned())
            ._handle(Method::POST, path, handler.to_owned())
            ._handle(Method::DELETE, path, handler.to_owned())
            ._handle(Method::PATCH, path, handler.to_owned())
            ._handle(Method::PUT, path, handler.to_owned())
            ._handle(Method::OPTIONS, path, handler.to_owned())
            ._handle(Method::HEAD, path, handler.to_owned())
            ._handle(Method::CONNECT, path, handler.to_owned())
            ._handle(Method::TRACE, path, handler.to_owned())
    }

    pub(crate) fn join_paths(a: &str, mut b: &str) -> String {
        if b.is_empty() {
            return a.to_owned();
        }
        a.trim_end_matches('/').to_owned() + "/" + b.trim_start_matches('/')
    }

    pub fn find<'a>(
        &'a self,
        method: &'a Method,
        path: &'a str,
    ) -> Option<(&'a H, Vec<(&'a str, &'a str)>)> {
        let tree = self.trees.get(method)?;
        tree.find(path)
    }
}
