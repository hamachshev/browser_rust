use std::{
    fs::File,
    io::{BufReader, Read, Write},
    net::TcpStream,
    sync::Arc,
};

use anyhow::Context;
use rustls::{ClientConfig, Stream};

use crate::{
    parser::HttpResponseParser,
    response::Response,
    url::{Scheme, URL},
};

const MAX_REDIRECTS: u32 = 10;

pub fn fetch(url: &URL) -> anyhow::Result<Response> {
    let mut response = request(url);

    for _ in 0..MAX_REDIRECTS {
        if let Ok(Response::Http(ref inner)) = response {
            let parsed = HttpResponseParser::parse(inner)?;
            if (300u32..400).contains(&parsed.status()) {
                let location = parsed
                    .headers_map()
                    .get("Location")
                    .ok_or(anyhow::anyhow!("Missing Location header in 300 response"))?;
                println!("{}", location);

                //handle relative paths
                let location = if location.starts_with("/") {
                    let scheme = url.scheme();
                    let base_url = url.host().context("relative path; couldnt get base")?;
                    format!("{}://{}{}", scheme, base_url, location)
                } else {
                    location.to_string()
                };

                println!("{}", location);
                let new_url: URL = location.parse()?;
                response = request(&new_url)
            } else {
                //got non redirect
                break;
            }
        } else {
            //non http
            break;
        }
    }

    //if still redirecting
    if let Ok(Response::Http(ref inner)) = response {
        let parsed = HttpResponseParser::parse(inner)?;
        if (300u32..400).contains(&parsed.status()) {
            return Err(anyhow::anyhow!("Too many redirects"));
        }
    }
    response
}
fn request(url: &URL) -> anyhow::Result<Response> {
    match url.scheme() {
        Scheme::Http => request_http(url),
        Scheme::Https => request_https(url),
        Scheme::File => request_file(url),
        Scheme::Data => request_data(url),
        Scheme::ViewSource => request_view_source(url),
        Scheme::Unknown => Err(anyhow::anyhow!("Cannot request unknown/unsupported scheme")),
    }
}

fn request_data(url: &URL) -> anyhow::Result<Response> {
    let data = url
        .data()
        .ok_or(anyhow::anyhow!("no data- should not happen"))?;
    Ok(Response::Data(data.to_string()))
}
fn request_file(url: &URL) -> anyhow::Result<Response> {
    let path = url
        .path()
        .ok_or(anyhow::anyhow!("missing path in file url"))?;
    println!("{}", path);
    let file = File::open(path)?;

    let mut bufread = BufReader::new(file);
    let mut contents = String::new();
    bufread.read_to_string(&mut contents)?;

    Ok(Response::File(contents))
}

fn request_view_source(url: &URL) -> anyhow::Result<Response> {
    let underlying_url: URL = url
        .data()
        .ok_or(anyhow::anyhow!("no data in view-source url"))?
        .parse()?;
    let res = request(&underlying_url)?;
    Ok(Response::ViewSource(Box::new(res)))
}

fn request_http(url: &URL) -> anyhow::Result<Response> {
    let host = url
        .host()
        .ok_or(anyhow::anyhow!("missing host in http request"))?;

    let path = url
        .path()
        .ok_or(anyhow::anyhow!("missing path in http request"))?;
    let mut stream = TcpStream::connect((host, 80))?;

    let request = format!(
        concat!(
            "GET {} HTTP/1.1\r\n",
            "Host: {}\r\n",
            "Connection: close\r\n",
            "User-Agent: browser_rust\r\n",
            "\r\n"
        ),
        path, host
    );

    println!("{}", &request);

    let _ = stream.write(request.as_bytes());
    let mut bufread = BufReader::new(&stream);
    let mut buffer = String::new();
    bufread.read_to_string(&mut buffer)?;
    Ok(Response::Http(buffer))
}

fn request_https(url: &URL) -> anyhow::Result<Response> {
    let host = url
        .host()
        .ok_or(anyhow::anyhow!("missing host in https request"))?;

    let path = url
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
                "User-Agent: browser_rust\r\n",
                "\r\n"
            ),
            path, host
        )
        .as_bytes(),
    )?;

    let mut buffer = String::new();
    let _ = stream.read_to_string(&mut buffer);
    Ok(Response::Http(buffer))
}
