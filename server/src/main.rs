#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use server::Server;
use website_handler::WebsiteHandler;

mod http;
mod server;
mod website_handler;

fn main() {
    let server_address: String = String::from("127.0.0.1:8080");
    let server: Server = Server::new(server_address);

    server.run(WebsiteHandler);
}
