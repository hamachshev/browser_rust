use std::{
    io::{BufReader, Read, Write},
    net::TcpStream,
    str::FromStr,
    sync::Arc,
    u32,
};

use rustls::{ClientConfig, Stream};
use thiserror::Error;

pub struct URL {
    serialization: String,
    scheme_end: usize, // does not include ://, and is exclusive ie one more than the actually
    // scheme
    host_end: usize,
    path_end: usize,
}

#[allow(unused)]
impl URL {
    pub fn scheme(&self) -> &str {
        &self.serialization[0..self.scheme_end]
    }

    pub fn host(&self) -> &str {
        &self.serialization[self.scheme_end + 3..self.host_end]
    }
    pub fn port(&self) -> Option<u32> {
        match self.host().split_once(":") {
            Some(port) => port.1.parse::<u32>().ok(),
            None => None,
        }
    }

    pub fn path(&self) -> &str {
        &self.serialization[self.host_end..self.path_end]
    }

    pub fn request(&self) -> anyhow::Result<String> {
        println!("Connecting to: {}", self.host());

        if self.scheme() == "https" {
            self.request_https()
        } else {
            self.request_http()
        }
    }
    fn request_http(&self) -> anyhow::Result<String> {
        let mut stream = TcpStream::connect((self.host(), 80))?;

        let request = format!(
            "GET {} HTTP/1.1 \r\nHost: {}\r\nConnection: close\r\n\r\n",
            self.path(),
            self.host()
        );

        println!("{}", &request);

        let _ = stream.write(request.as_bytes());
        let mut bufread = BufReader::new(&stream);
        let mut buffer = String::new();
        bufread.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    fn request_https(&self) -> anyhow::Result<String> {
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let rc_config = Arc::new(config);
        let mut client =
            rustls::ClientConnection::new(rc_config, self.host().to_string().try_into()?)?;
        let mut socket = TcpStream::connect((self.host(), 443))?;
        let mut stream = Stream::new(&mut client, &mut socket);

        stream.write_all(
            format!(
                concat!(
                    "GET {} HTTP/1.1\r\n",
                    "Host: {}\r\n",
                    "Connection: close\r\n",
                    "\r\n"
                ),
                self.path(),
                self.host()
            )
            .as_bytes(),
        );

        let mut buffer = String::new();
        let _ = stream.read_to_string(&mut buffer);
        Ok(buffer)
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

    #[error("non http scheme")]
    NonHTTPScheme,
}

impl FromStr for URL {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let mut s = s.to_string();

        let scheme_end = s.find("://").ok_or(ParseError::SchemeMissing)?;
        println!("{}", &s[0..scheme_end]);

        if &s[0..scheme_end] != "http" && &s[0..scheme_end] != "https" {
            return Err(ParseError::NonHTTPScheme);
        }
        let after_scheme = scheme_end + 3;

        if s[after_scheme..].is_empty() {
            return Err(ParseError::HostMissing);
        }
        let host_end = match s[after_scheme..].find("/") {
            Some(end) => after_scheme + end,
            None => {
                s.push('/');
                s.len() - 1
            } //end of host is end of string
        };

        println!("host_end {}", &s[after_scheme..host_end]);
        let path_end = if host_end == s.len() {
            host_end
        } else {
            s.len()
        };
        Ok(Self {
            serialization: s,
            scheme_end,
            host_end,
            path_end,
        })
    }
}
