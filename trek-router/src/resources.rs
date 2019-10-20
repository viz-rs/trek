use http::Method;

#[derive(Debug)]
pub enum Resource {
    Show,
    Create,
    Update(Method),
    Delete,
    Edit,
    New,
}

impl Resource {
    pub fn as_tuple(&self) -> (&str, Method) {
        match self {
            Self::Show => ("", Method::GET),
            Self::Create => ("", Method::POST),
            Self::Update(method) => ("", method.to_owned()),
            Self::Delete => ("", Method::DELETE),
            Self::Edit => ("edit", Method::GET),
            Self::New => ("new", Method::GET),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum Resources {
    Index,
    Create,
    New,
    Show,
    Update(Method),
    Delete,
    Edit,
}

impl Resources {
    pub fn as_tuple(&self) -> (&str, Method) {
        match self {
            Self::Index => ("", Method::GET),
            Self::Create => ("", Method::POST),
            Self::New => ("new", Method::GET),
            Self::Show => (":id", Method::GET),
            Self::Update(method) => (":id", method.to_owned()),
            Self::Delete => (":id", Method::DELETE),
            Self::Edit => (":id/edit", Method::GET),
            _ => unimplemented!(),
        }
    }
}
