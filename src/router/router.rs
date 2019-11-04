use fnv::FnvHashMap;
use http::Method;
use inflector::string::{pluralize::to_plural, singularize::to_singular};
use path_tree::PathTree;
use std::{fmt, mem, sync::Arc};

use crate::engine::handler::box_dyn_handler_into_middleware;
use crate::engine::handler::into_box_dyn_handler;
// use crate::engine::handler::into_middleware;
use crate::engine::handler::BoxDynHandler;
use crate::engine::handler::Handler;
use crate::engine::middleware::Middleware;
use crate::router::resource::{Resource, Resources};

pub type Trees<Context> = FnvHashMap<Method, PathTree<Vec<Arc<dyn Middleware<Context>>>>>;

pub struct Router<Context> {
    path: String,
    trees: Trees<Context>,
    middleware: Vec<Arc<dyn Middleware<Context>>>,
}

impl<Context: Send + 'static> Router<Context> {
    pub fn new() -> Self {
        Self {
            path: "/".to_owned(),
            trees: Trees::default(),
            middleware: Vec::new(),
        }
    }

    pub fn middleware(&mut self, m: impl Middleware<Context>) -> &mut Self {
        self.middleware.push(Arc::new(m));
        self
    }

    pub fn clear_middleware(&mut self) -> &mut Self {
        self.middleware.clear();
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

        mem::swap(&mut self.trees, &mut router.trees);

        self
    }

    fn _handle(
        &mut self,
        path: &str,
        method: Method,
        handler: BoxDynHandler<Context>,
    ) -> &mut Self {
        let path = &Self::join_paths(&self.path, path);
        let mut middleware = self.middleware.clone();
        middleware.push(Arc::new(box_dyn_handler_into_middleware(handler)));

        info!("route: {} {}", method, path);

        self.trees
            .entry(method)
            .or_insert_with(PathTree::new)
            .insert(path, middleware);

        self
    }

    pub fn handle(
        &mut self,
        path: &str,
        method: Method,
        h: impl Handler<Context> + Clone,
    ) -> &mut Self {
        self._handle(path, method, into_box_dyn_handler(h))
    }

    pub fn get(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::GET, h)
    }

    pub fn post(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::POST, h)
    }

    pub fn delete(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::DELETE, h)
    }

    pub fn patch(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::PATCH, h)
    }

    pub fn put(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::PUT, h)
    }

    pub fn options(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::OPTIONS, h)
    }

    pub fn head(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::HEAD, h)
    }

    pub fn connect(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::CONNECT, h)
    }

    pub fn trace(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::TRACE, h)
    }

    pub fn any(&mut self, path: &str, h: impl Handler<Context> + Clone) -> &mut Self {
        self.handle(path, Method::GET, h.clone())
            .handle(path, Method::POST, h.clone())
            .handle(path, Method::DELETE, h.clone())
            .handle(path, Method::PATCH, h.clone())
            .handle(path, Method::PUT, h.clone())
            .handle(path, Method::OPTIONS, h.clone())
            .handle(path, Method::HEAD, h.clone())
            .handle(path, Method::CONNECT, h.clone())
            .handle(path, Method::TRACE, h)
    }

    pub fn resource(
        &mut self,
        path: &str,
        maps: &[(Resource, BoxDynHandler<Context>)],
    ) -> &mut Self {
        let s = if path.is_empty() {
            self.path.rsplitn(2, '/').collect::<Vec<&str>>()[0]
        } else {
            path
        };
        let path = &to_singular(s);
        for (resource, handler) in maps {
            let (sub_path, method) = resource.as_tuple();
            let path = &Self::join_paths(&path, sub_path);
            self._handle(path, method, handler.clone());
        }
        self
    }

    pub fn resources(
        &mut self,
        path: &str,
        maps: &[(Resources, BoxDynHandler<Context>)],
    ) -> &mut Self {
        let (p, s) = if path.is_empty() {
            (
                "".to_owned(),
                self.path.rsplitn(2, '/').collect::<Vec<&str>>()[0],
            )
        } else {
            (to_plural(path), path)
        };
        let spath = to_singular(s);
        for (resources, handler) in maps {
            let (sub_path, method) = resources.as_tuple();
            let path = &Self::join_paths(&p, &sub_path.replace("id", &(spath.to_owned() + "_id")));
            self._handle(path, method, handler.clone());
        }
        self
    }

    pub fn find<'a>(
        &'a self,
        path: &'a str,
        method: Method,
    ) -> Option<(
        &'a Vec<Arc<dyn Middleware<Context>>>,
        Vec<(&'a str, &'a str)>,
    )> {
        let tree = self.trees.get(&method)?;
        tree.find(path)
    }

    pub(crate) fn join_paths(a: &str, b: &str) -> String {
        if b.is_empty() {
            return a.to_owned();
        }
        a.trim_end_matches('/').to_owned() + "/" + b.trim_start_matches('/')
    }
}

impl<Context> fmt::Debug for Router<Context> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Router")
            .field("path", &self.path)
            .finish()
    }
}
