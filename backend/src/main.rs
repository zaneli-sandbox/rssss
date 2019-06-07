pub mod error;
pub mod rss;

use actix_web::client::{Client, ClientResponse, SendRequestError};
use actix_web::error::PayloadError;
use actix_web::middleware::cors::Cors;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use bytes::Bytes;
use futures::future;
use futures::{Future, Stream};
use listenfd::ListenFd;
use log::info;
use regex::Regex;
use std::env;
use std::time::Duration;

impl From<error::Error> for HttpResponse {
    fn from(e: error::Error) -> HttpResponse {
        HttpResponse::BadRequest().json(e)
    }
}

fn get_feed(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = Error> {
    let reg = Regex::new(r"\&?url=(.+)\&?").unwrap();
    let url = reg.captures(req.query_string()).unwrap();
    send_request(&url[1])
        .map_err(Error::from)
        .and_then(|r| retrieve_response(r, 3))
}

fn send_request(
    url: &str,
) -> impl Future<
    Item = ClientResponse<impl Stream<Item = Bytes, Error = PayloadError>>,
    Error = SendRequestError,
> {
    info!("{}", url);
    let client = Client::default();
    client
        .get(url)
        .header("User-Agent", "rssss")
        .timeout(Duration::from_secs(60))
        .send()
}

fn retrieve_response(
    mut res: ClientResponse<impl Stream<Item = Bytes, Error = PayloadError> + 'static>,
    redirect_limit: u8,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let status = res.status();
    if status.is_success() {
        Box::new(
            res.body()
                .limit(1_048_576) // 1MB
                .from_err()
                .and_then(|b| match rss::parse_rss(b) {
                    Ok(r) => Ok(HttpResponse::Ok().json(r)),
                    Err(e) => Ok(e.into()),
                }),
        )
    } else if status.is_redirection() && redirect_limit > 0 {
        match res.headers().get("location").and_then(|l| l.to_str().ok()) {
            Some(url) => Box::new(
                send_request(&url.to_string())
                    .map_err(Error::from)
                    .and_then(move |r| retrieve_response(r, redirect_limit - 1)),
            ),
            _ => Box::new(future::ok::<HttpResponse, Error>(
                HttpResponse::InternalServerError().finish(),
            )),
        }
    } else {
        Box::new(future::ok::<HttpResponse, Error>(
            HttpResponse::build(res.status()).finish(),
        ))
    }
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(Cors::new())
            .service(web::resource("/feed").route(web::get().to_async(get_feed)))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        let host = env::var("RSSSS_BACKEND_HOST").unwrap_or("localhost".to_string());
        let port = env::var("RSSSS_BACKEND_PORT")
            .unwrap_or("8080".to_string())
            .parse::<u32>()
            .unwrap();
        server.bind(format!("{}:{}", host, port)).unwrap()
    };

    server.run().unwrap();
}
