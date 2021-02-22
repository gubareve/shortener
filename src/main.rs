use colored::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

#[derive(Deserialize, Debug)]
struct FirebaseResponse {
    shortLink: Option<String>,
    warning: Option<[HashMap<String, String>; 1]>,
    previewLink: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api_key_file = match File::open(&Path::new("/etc/dynamic-links.conf")) {
        Ok(file) => file,
        Err(error) => {
            println!(
                "{}",
                "Please place your firebase web-api key in /etc/dynamic-links.conf"
                    .red()
                    .bold()
            );
            process::exit(1);
        }
    };
    let mut api_key = String::new();
    api_key_file
        .read_to_string(&mut api_key)
        .expect("Please place your firebase web-api key in /etc/dynamic-links.conf");
    if api_key.ends_with('\n') {
        api_key.pop();
        if api_key.ends_with('\r') {
            api_key.pop();
        }
    }

    let args: Vec<String> = env::args().collect();
    let mode: String;

    if args.len() <= 1 {
        eprintln!(
            "{}",
            "You must specify the url you wish to minify".red().bold()
        );
        process::exit(1);
    }

    if args.len() > 3 {
        eprintln!("{}", "Too many args".red().bold());
        process::exit(1);
    }

    let mut modes = HashMap::new();
    modes.insert("short", "SHORT");
    modes.insert("long", "UNGUESSABLE");

    if args.len() == 3 {
        if modes.contains_key(&args[2] as &str) {
            match modes.get(&args[2] as &str) {
                Some(x) => mode = x.to_string(),
                None => process::exit(1),
            }
        } else {
            eprintln!(
                "{}",
                format!(
                    "{}{}{}{}",
                    "Invalid mode, either specify ".red(),
                    "long".blue().bold(),
                    " or ".red(),
                    "short".yellow().bold()
                )
            );
            process::exit(1);
        }
    } else {
        mode = "short".to_string();
    }

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&format!(
            "https://firebasedynamiclinks.googleapis.com/v1/shortLinks?key={}",
            api_key
        ))
        .body(format!(
            "{{\"longDynamicLink\":\"https://f.evang.dev?link={}\",\"suffix\":{{\"option\":\"{}\"}}}}",
            args[1], mode
        ))
        .send()?;

    let response_text = resp.text()?;
    let response_text_error = response_text.clone();

    let response: FirebaseResponse = serde_json::from_str(&String::from(response_text)).unwrap();

    let mut shortLink: String = match response.shortLink {
        None => format!(
            "🚨{}🚨\n{}",
            "Could not find the output url".red().bold(),
            response_text_error
        ),
        Some(link) => format!("Got shortened link: {}", link.yellow().bold()),
    };
    println!("{}", shortLink);
    Ok(())
}
