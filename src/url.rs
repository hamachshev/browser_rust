use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::TcpStream,
    str::FromStr,
    sync::Arc,
};

use crate::response::Response;
use rustls::{ClientConfig, Stream};
use thiserror::Error;

const ALLOWED_SCHEMES: [&'static str; 4] = ["http", "https", "file", "data"];
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
    pub fn scheme(&self) -> &str {
        &self.serialization[0..self.scheme_end]
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

    pub fn request(&self) -> anyhow::Result<Response> {
        if let Some(host) = self.host() {
            println!("Connecting to: {}", host);
        };

        match self.scheme() {
            "https" => self.request_https(),
            "http" => self.request_http(),
            "file" => self.request_file(),
            _ => Ok(Response::None),
        }
    }
    fn data(&self) -> Option<&str> {
        if let Some(data_end) = self.data_end {
            Some(&self.serialization[self.scheme_end + 1..data_end])
        } else {
            None
        }
    }
    fn request_file(&self) -> anyhow::Result<Response> {
        let path = self
            .path()
            .ok_or(anyhow::anyhow!("missing path in file url"))?;
        println!("{}", path);
        let file = File::open(path)?;

        let mut bufread = BufReader::new(file);
        let mut contents = String::new();
        bufread.read_to_string(&mut contents);

        Ok(Response::File(contents))
    }

    fn request_http(&self) -> anyhow::Result<Response> {
        let host = self
            .host()
            .ok_or(anyhow::anyhow!("missing host in http request"))?;

        let path = self
            .path()
            .ok_or(anyhow::anyhow!("missing path in http request"))?;
        let mut stream = TcpStream::connect((host, 80))?;

        let request = format!(
            "GET {} HTTP/1.1 \r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host
        );

        println!("{}", &request);

        let _ = stream.write(request.as_bytes());
        let mut bufread = BufReader::new(&stream);
        let mut buffer = String::new();
        bufread.read_to_string(&mut buffer)?;
        Ok(Response::Http(buffer))
    }

    fn request_https(&self) -> anyhow::Result<Response> {
        let host = self
            .host()
            .ok_or(anyhow::anyhow!("missing host in https request"))?;

        let path = self
            .path()
            .ok_or(anyhow::anyhow!("missing path in https request"))?;

        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let rc_config = Arc::new(config);
        let mut client = rustls::ClientConnection::new(rc_config, host.to_string().try_into()?)?;
        let mut socket = TcpStream::connect((host, 443))?;
        let mut stream = Stream::new(&mut client, &mut socket);

        stream.write_all(
            format!(
                concat!(
                    "GET {} HTTP/1.1\r\n",
                    "Host: {}\r\n",
                    "Connection: close\r\n",
                    "\r\n"
                ),
                path, host
            )
            .as_bytes(),
        );

        let mut buffer = String::new();
        let _ = stream.read_to_string(&mut buffer);
        Ok(Response::Http(buffer))
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

        assert_eq!(url.scheme(), "http");
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
        assert_eq!(url.scheme(), "https")
    }

    #[test]
    fn data_url() {
        let url: URL = "data:text/html,Hello World!".parse().unwrap();
        assert_eq!(url.scheme(), "data");
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), None);
        assert_eq!(url.data(), Some("text/html,Hello World!"));
    }
    #[test]
    fn file_url_with_blank_authority() {
        let url: URL = "file:///cargo.toml".parse().unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.host(), Some(""));
        assert_eq!(url.path(), Some("/cargo.toml"));
        assert_eq!(url.data(), None);
    }

    #[test]
    fn file_url_with_no_authority() {
        let url: URL = "file:/cargo.toml".parse().unwrap();
        assert_eq!(url.scheme(), "file");
        assert_eq!(url.host(), None);
        assert_eq!(url.path(), Some("/cargo.toml"));
        assert_eq!(url.data(), None);
    }

    #[test]
    fn http_with_port() {
        let url: URL = "http://localhost:8080/index.html".parse().unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), Some("localhost:8080"));
        assert_eq!(url.port(), Some(8080))
    }
}
