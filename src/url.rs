use std::{fmt::Display, str::FromStr};

use thiserror::Error;

const ALLOWED_SCHEMES: [&'static str; 5] = ["http", "https", "file", "data", "view-source"];

#[derive(PartialEq, Eq, Debug)]
pub enum Scheme {
    Http,
    Https,
    File,
    Data,
    ViewSource,
    Unknown,
}
impl Display for Scheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scheme = match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
            Scheme::File => "file",
            Scheme::Data => "data",
            Scheme::ViewSource => "view-source",
            Scheme::Unknown => "",
        };
        write!(f, "{}", scheme)
    }
}
pub struct URL {
    serialization: String,
    scheme_end: usize, // does not include ://, and is exclusive ie one more than the actually
    // scheme
    host_end: Option<usize>,
    path_end: Option<usize>,
    data_end: Option<usize>,
}

#[allow(unused)]
impl URL {
    pub fn scheme(&self) -> Scheme {
        match &self.serialization[0..self.scheme_end] {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            "file" => Scheme::File,
            "data" => Scheme::Data,
            "view-source" => Scheme::ViewSource,
            _ => Scheme::Unknown,
        }
    }

    pub fn host(&self) -> Option<&str> {
        if let Some(host_end) = self.host_end {
            //skip ://
            Some(&self.serialization[self.scheme_end + 3..host_end])
        } else {
            None
        }
    }
    pub fn port(&self) -> Option<u32> {
        let Some(host) = self.host() else {
            return None;
        };
        match host.split_once(":") {
            Some(port) => port.1.parse::<u32>().ok(),
            None => None,
        }
    }

    pub fn path(&self) -> Option<&str> {
        let path_end = self.path_end?;

        if let Some(host_end) = self.host_end {
            Some(&self.serialization[host_end..path_end])
        } else {
            //can have path without host ie file:/hello.png
            Some(&self.serialization[self.scheme_end + 1..path_end])
        }
    }

    pub fn data(&self) -> Option<&str> {
        if let Some(data_end) = self.data_end {
            Some(&self.serialization[self.scheme_end + 1..data_end])
        } else {
            None
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    //pub for some reason
    #[error("Cannot parse empty string")]
    Empty,
    #[error("missing scheme")]
    SchemeMissing,

    #[error("missing host")]
    HostMissing,

    #[error("unknown scheme")]
    UnknownScheme,
}

impl FromStr for URL {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let mut s = s.to_string();

        let scheme_end = s.find(":").ok_or(ParseError::SchemeMissing)?;

        let scheme = &s[0..scheme_end];
        if !ALLOWED_SCHEMES.contains(&scheme) {
            return Err(ParseError::UnknownScheme);
        }

        let mut host_end = None; // start assuming there is no host
        let mut path_end = None; // and that there is no path
        let mut data_end = None; // and there is no data

        if let Some(b'/') = s.as_bytes().get(scheme_end + 1) {
            //there is a path, so heirarchial
            if s.get(scheme_end + 1..scheme_end + 3) == Some("//") {
                // there is an authority/host
                let after_scheme = scheme_end + 3;

                if s[after_scheme..].is_empty() {
                    return Err(ParseError::HostMissing);
                }

                host_end = match s[after_scheme..].find("/") {
                    Some(end) => Some(after_scheme + end),
                    None => {
                        s.push('/');
                        Some(s.len() - 1)
                    } //end of host is end of string
                };
            }
            path_end = Some(s.len());
        } else {
            //opaque scheme
            data_end = Some(s.len());
        }

        Ok(Self {
            serialization: s,
            scheme_end,
            host_end,
            path_end,
            data_end,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn http_url() {
        let url = "http://www.google.com".parse::<URL>().unwrap();

        assert_eq!(url.scheme(), Scheme::Http);
        assert_eq!(url.host(), Some("www.google.com"));
        assert_eq!(url.path(), Some("/"));
        assert_eq!(url.data(), None);
    }
    #[test]
    fn http_url_trailing_slash() {
        let url = "http://www.google.com/".parse::<URL>().unwrap();

        assert_eq!(url.path(), Some("/"));
    }
    #[test]
    fn http_url_no_trailing_slash() {
        let url = "http://www.google.com".parse::<URL>().unwrap();

        assert_eq!(url.path(), Some("/"));
    }
    #[test]
    fn https_url() {
        let url = "https://www.google.com".parse::<URL>().unwrap();
        assert_eq!(url.scheme(), Scheme::Https)
    }

    #[test]
    fn data_url() {
        let url: URL = "data:text/html,Hello World!".parse().unwrap();
        assert_eq!(url.scheme(), Scheme::Data);
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), None);
        assert_eq!(url.data(), Some("text/html,Hello World!"));
    }
    #[test]
    fn file_url_with_blank_authority() {
        let url: URL = "file:///cargo.toml".parse().unwrap();
        assert_eq!(url.scheme(), Scheme::File);
        assert_eq!(url.host(), Some(""));
        assert_eq!(url.path(), Some("/cargo.toml"));
        assert_eq!(url.data(), None);
    }

    #[test]
    fn file_url_with_no_authority() {
        let url: URL = "file:/cargo.toml".parse().unwrap();
        assert_eq!(url.scheme(), Scheme::File);
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), Some("/cargo.toml"));
        assert_eq!(url.data(), None);
    }

    #[test]
    fn http_with_port() {
        let url: URL = "http://localhost:8080/index.html".parse().unwrap();
        assert_eq!(url.scheme(), Scheme::Http);
        assert_eq!(url.host(), Some("localhost:8080"));
        assert_eq!(url.port(), Some(8080))
    }

    #[test]
    fn view_source_http() {
        let url: URL = "view-source:http://browser.engineering/examples/example1-simple.html"
            .parse()
            .unwrap();
        assert_eq!(url.scheme(), Scheme::ViewSource);
        assert_eq!(url.host(), None);
        assert_eq!(
            url.data(),
            Some("http://browser.engineering/examples/example1-simple.html")
        )
    }
}
