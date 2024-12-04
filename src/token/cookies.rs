use crate::config::ENV;
use axum::http::{header, HeaderMap};
use cookie::{time::Duration, Cookie, CookieBuilder};
use envmode::EnvMode;

pub struct CookieManager;

#[derive(Default)]
pub struct CookieParams<'a> {
    pub age: Option<usize>,
    pub http_only: Option<bool>,
    pub domain: Option<&'a str>,
    pub path: Option<&'a str>,
}

impl<'a> CookieParams<'a> {
    pub fn with_age(mut self, age: usize) -> Self {
        self.age = Some(age);
        self
    }

    pub fn with_http_only(mut self, http_only: bool) -> Self {
        self.http_only = Some(http_only);
        self
    }

    pub fn with_domain(mut self, domain: &'a str) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn with_path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }
}

impl CookieManager {
    pub fn create<'a>(
        name: &'a str,
        value: &'a str,
        params: CookieParams<'a>,
    ) -> CookieBuilder<'a> {
        let mut builder = Cookie::build((name, value)).secure(EnvMode::is_prd(&ENV.env));

        if let Some(age) = params.age {
            builder = builder.max_age(Duration::seconds(age as i64));
        }
        if let Some(http_only) = params.http_only {
            builder = builder.http_only(http_only);
        }
        if let Some(domain) = params.domain {
            builder = builder.domain(domain);
        } else {
            builder = builder.domain(&*ENV.domain);
        }
        if let Some(path) = params.path {
            builder = builder.path(path);
        } else {
            builder = builder.path("/");
        }

        builder
    }

    pub fn get<'a>(headers: &'a HeaderMap, name: &str) -> Option<Cookie<'a>> {
        headers
            .get(header::COOKIE)
            .and_then(|ck| ck.to_str().ok())
            .and_then(|ck| {
                ck.split(';')
                    .filter_map(|c| Cookie::parse(c.to_owned()).ok())
                    .find(|c| c.name() == name)
            })
    }

    pub fn delete<'a>(name: &'a str, params: CookieParams<'a>) -> CookieBuilder<'a> {
        Self::create(name, "", params.with_age(0))
    }
}
