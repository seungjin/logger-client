use anyhow::{Error, Result};
use clap::{Parser, ValueEnum};
use reqwest::Client;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str;
use tokio::io;
use tokio::io::Interest;
use tokio::net::UnixListener;
use tokio::net::UnixSocket;
use tokio::net::UnixStream;
use tokio::signal;
use users::{get_current_uid, get_user_by_uid};

/// Logger client
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(group(
    clap::ArgGroup::new("input")
        .required(true)
        .args(&["sock", "pipe"]),
))]
struct Args {
    #[arg(short, long)]
    sock: Option<String>,

    #[arg(short, long)]
    pipe: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.sock {
        Some(k) => socket_interface(k).await,
        None => (),
    };

    match args.pipe {
        Some(k) => pipe_interface(k).await,
        None => (),
    }
}

async fn socket_interface(mut key: String) {
    println!("Socket interface selected");

    if key.ends_with("/") {
        println!("Soket shoud be a file not dirctory: {}", key);
        std::process::exit(0);
    }

    if key.starts_with("/") {
        key.remove(0);
    }

    let uid = get_current_uid().to_string();
    let socket_path = Path::new("/run/user/")
        .join(uid)
        .join("seungjin-logger")
        .join(&key);

    let parent_dir = socket_path.parent().unwrap();

    if !socket_path.exists() {
        fs::create_dir_all(&parent_dir).expect(
            format!("Can't create socket path: {}", parent_dir.display())
                .as_str(),
        );
    };

    println!("Local socket path: {}", &socket_path.display());

    let hostname = hostname::get().unwrap().to_str().unwrap().to_owned();
    let auth_key = match env::var("LOGGER_AUTHKEY") {
        Ok(v) => v,
        Err(e) => panic!("$LOGGER_AUTHKEY is not set ({})", e),
    };
    let endpoint = format!("https://logger.seungjin.net/{}/{}", hostname, key);
    println!("Remote log endpoint: {}", endpoint);

    let client = reqwest::Client::new();
    let res = client
        .post(&endpoint)
        .header("AUTHKEY", &auth_key)
        .header("Context-Type", "application/json");

    println!("{:?}", res);

    let listener =
        UnixListener::bind(&socket_path.to_str().expect("arsars")).unwrap();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nUser quites: <Ctrl+C> received.");
        fs::remove_file(&socket_path);
        println!("Socket removed: {}", socket_path.display());
        std::process::exit(0);
    });

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                process_stream(stream, &client, &auth_key, &key, &hostname)
                    .await
            }
            Err(e) => { /* connection failed */ } // Todo:
        }
    }
}

async fn pipe_interface(key: String) {
    let stdin = std::io::stdin();
    let mut message = String::new();
    for line in stdin.lock().lines() {
        let line = line.expect("Could not read line from standard in");
        //println!("{}", line);
        message.push_str(line.as_str());
        message.push_str("\n");
    }

    let hostname = hostname::get().unwrap().to_str().unwrap().to_owned();
    let auth_key = match env::var("LOGGER_AUTHKEY") {
        Ok(v) => v,
        Err(e) => panic!("$LOGGER_AUTHKEY is not set ({})", e),
    };
    let endpoint = format!("https://logger.seungjin.net/{}/{}", hostname, key);
    println!("Remote log endpoint: {}", endpoint);

    let client = reqwest::Client::new();
    let res = client
        .post(&endpoint)
        .header("AUTHKEY", &auth_key)
        .header("Context-Type", "application/json")
        .body(message)
        .send()
        .await;
}

async fn process_stream(
    stream: UnixStream,
    client: &Client,
    auth_key: &String,
    key: &String,
    hostname: &String,
) {
    loop {
        // Wait for the socket to be readable
        stream.readable().await;

        let mut buf = Vec::with_capacity(4096);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("read {} bytes", n);
                let s = match String::from_utf8(buf) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e), // Todo:
                };
                post_message(client, auth_key, hostname, key, s).await;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("rasras"); // Todo:
                continue;
            }
            Err(e) => {
                //return Err(e.into());
                // Todo:
                eprintln!("Error from stream.try_read_buffer");
            }
        }
    }
}

async fn post_message(
    client: &Client,
    auth_key: &String,
    hostname: &String,
    key: &String,
    message: String,
) -> Result<()> {
    let endpoint = format!("https://logger.seungjin.net/{}/{}", hostname, key);

    let res = client
        .post(&endpoint)
        .header("AUTHKEY", auth_key)
        .header("Context-Type", "application/json")
        .body(message)
        .send()
        .await?;

    println!("{:?}", res);

    Ok(())
}
