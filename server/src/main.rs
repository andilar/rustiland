

fn main() {
    let string: String = String::from("127.0.0.1:8080");
    let string_slice: &str = &string[10..];

    let server: Server = Server::new(string);
    server.run();
}

struct Server{
    addr: String,
}

impl Server{
    fn new(addr: String)-> Self {
        Server{
            addr
        }
    }

    fn run(self){

    }
}