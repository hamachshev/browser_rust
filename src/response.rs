use std::fmt::Display;

pub enum Response {
    Http(String),
    File(String),
    None,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Http(res) => write!(f, "{}", res),
            Response::File(res) => write!(f, "{}", res),
            Response::None => write!(f, "No response"),
        }
    }
}
