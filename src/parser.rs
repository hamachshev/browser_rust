use std::{collections::HashMap, fmt::Display, rc::Rc};
use thiserror::Error;

use crate::response::Response;

pub fn parse(response: Response) -> anyhow::Result<Box<dyn Display>> {
    match response {
        Response::Http(res) => {
            let res = Rc::new(res);
            Ok(Box::new(HttpResponseParser::parse(&res)?))
        }
        Response::File(res) => Ok(Box::new(res)),
        Response::None => todo!(),
    }
}
#[derive(Debug, Error)]
pub enum HttpResponseParseError {
    #[error("missing http version in response")]
    MissingHTTPVersion,

    #[error("missing status number in response")]
    MissingStatus,

    #[error("missing status message in response")]
    MissingStatusMessage,

    #[error("malformed header in response")]
    MalformedHeader,
}
pub struct HttpResponseParser {
    http_version: String,
    status: u32,
    message: String,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponseParser {
    pub fn parse(response: &str) -> Result<HttpResponseParser, HttpResponseParseError> {
        let (http_version, rest) = response
            .split_once(" ")
            .ok_or(HttpResponseParseError::MissingHTTPVersion)?;
        let (status, rest) = rest
            .split_once(" ")
            .ok_or(HttpResponseParseError::MissingStatus)?;
        let status = status
            .parse::<u32>()
            .map_err(|_| HttpResponseParseError::MissingStatus)?;
        let (message, rest) = rest
            .split_once("\r\n")
            .ok_or(HttpResponseParseError::MissingStatusMessage)?;
        let mut headers = HashMap::new();

        let (raw_headers, body) = rest
            .split_once("\r\n\r\n")
            .ok_or(HttpResponseParseError::MalformedHeader)?;

        for line in raw_headers.split("\r\n") {
            if line.is_empty() {
                // last line is empty in http
                break;
            }
            let (key, value) = line
                .split_once(":")
                .ok_or(HttpResponseParseError::MalformedHeader)?;
            headers.insert(key.to_string(), value.to_string());
        }

        Ok(HttpResponseParser {
            http_version: http_version.to_string(),
            status,
            message: message.to_string(),
            headers,
            body: body.to_string(),
        })
    }

    pub fn status(&self) -> u32 {
        self.status
    }
    pub fn body(&self) -> &str {
        self.body.as_str()
    }
    pub fn status_message(&self) -> &str {
        self.message.as_str()
    }
    pub fn headers_map(&self) -> &HashMap<String, String> {
        &self.headers
    }
    pub fn http_version(&self) -> &str {
        self.http_version.as_str()
    }
}

impl Display for HttpResponseParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", HTMLParser::parse(self.body()))
    }
}

pub struct HTMLParser;

impl HTMLParser {
    pub fn parse(body: &str) -> String {
        let mut in_tag = false;
        let mut text = String::new();
        for c in body.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                c => {
                    if !in_tag {
                        text.push(c)
                    }
                }
            }
        }

        return text;
    }
}
