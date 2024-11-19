#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use http::Method;
use http::Request;

mod server;
use server::Server;

mod http;

fn main() {
    let server_address: String = String::from("127.0.0.1:8080");
    let server: Server = Server::new(server_address);

    server.run();
}
