use std::collections::HashMap;
use thiserror::Error;

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
pub struct HttpResponseParser<'a> {
    response: &'a str,
    http_version: &'a str,
    status: u32,
    message: &'a str,
    headers: HashMap<&'a str, &'a str>,
    body: &'a str,
}

impl<'a> HttpResponseParser<'a> {
    pub fn parse(response: &str) -> Result<HttpResponseParser<'_>, HttpResponseParseError> {
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
            headers.insert(key, value);
        }

        Ok(HttpResponseParser {
            response,
            http_version,
            status,
            message,
            headers,
            body,
        })
    }

    pub fn status(&self) -> u32 {
        self.status
    }
    pub fn body(&self) -> &str {
        self.body
    }
    pub fn status_message(&self) -> &str {
        self.message
    }
    pub fn headers_map(&self) -> &HashMap<&str, &str> {
        &self.headers
    }
    pub fn http_version(&self) -> &str {
        self.http_version
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
