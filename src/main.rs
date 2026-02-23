use std::{env::args, process::exit};

use browser_rust::{engine::fetch, parser::parse, url::URL};

fn main() -> anyhow::Result<()> {
    let Some(url) = args().nth(1) else {
        eprintln!("Must supply a url as cmd line arg");
        exit(1);
    };
    let url: URL = url.parse()?;
    let res = fetch(&url)?;
    let parsed = parse(res)?;

    print!("{}", &parsed);
    //let parser = HttpResponseParser::parse(&res)?;
    //let html = HTMLParser::parse(&parser.body());
    //println!("{}", html);
    //
    Ok(())
}
