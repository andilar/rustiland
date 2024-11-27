#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use server::Server;
use std::{default, env};
use website_handler::WebsiteHandler;

mod http;
mod server;
mod website_handler;

fn main() {
    let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));//backaslash?
    let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);

    println!("public path {}", public_path);
    let server_address: String = String::from("127.0.0.1:8080");
    let server: Server = Server::new(server_address);

    server.run(WebsiteHandler::new(public_path));
}
