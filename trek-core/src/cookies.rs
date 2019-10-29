use cookie::{Cookie, CookieJar, ParseError};
use http::HeaderMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Cookies(pub(crate) Arc<RwLock<CookieJar>>);

impl Cookies {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self(Arc::new(RwLock::new(
            parse_from_header(
                headers
                    .get_all(http::header::COOKIE)
                    .iter()
                    .map(|raw| raw.to_str().unwrap())
                    .collect(),
            )
            .unwrap_or_default(),
        )))
    }
}

fn parse_from_header(cs: Vec<&str>) -> Result<CookieJar, ParseError> {
    let mut jar = CookieJar::new();

    for s in cs {
        s.split(';').try_for_each(|s| -> Result<_, ParseError> {
            jar.add_original(Cookie::parse(s.trim().to_owned())?);
            Ok(())
        })?;
    }

    Ok(jar)
}
