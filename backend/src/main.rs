extern crate actix_web;
extern crate listenfd;

use actix_web::{http, server, App, Path, Responder};
use listenfd::ListenFd;

fn index(info: Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", info.1, info.0)
}

fn main() {
    let mut listenfd = ListenFd::from_env();

    let mut server =
        server::new(|| App::new().route("/{id}/{name}/index.html", http::Method::GET, index));

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
}
