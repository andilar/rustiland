use http::request::Request;
use http::method::Method;

mod server;
use server::Server;

mod http;


fn main() {
    let server_address: String = String::from("127.0.0.1:8080");
    let server: Server = Server::new(server_address);

    server.run();
}
