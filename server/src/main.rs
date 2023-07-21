

fn main() {
let get = Method::GET;
let delete = Method::DELETE;
let post = Method::POST;
let put = Method::PUT;

    let server_address: String = String::from("127.0.0.1:8080");
    let server: Server = Server::new(server_address);

    server.run();
}

struct Server{
    addr: String,
}

impl Server{
    fn new(addr: String)-> Self {
        Self { addr }
    }

    fn run(self){
        println!("Listening on {}", self.addr);

    }
}

struct Request {
    path: String,
    query_string: String,
    method: Method,
}

enum Method {
    GET,
    DELETE,
    POST,
    PUT,
    HEAD,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}
