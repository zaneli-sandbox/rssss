pub mod error;
pub mod rss;

use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::web::Query;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use awc::SendClientRequest;
use listenfd::ListenFd;
use log::info;
use serde::Serialize;
use serde_derive::Deserialize;
use std::env;
use std::io;
use std::time::Duration;

#[derive(Deserialize)]
struct Info {
    url: String,
}

impl<T: Serialize> From<error::Error<T>> for HttpResponse {
    fn from(e: error::Error<T>) -> HttpResponse {
        HttpResponse::BadRequest().json(e)
    }
}

async fn get_feed(info: Query<Info>) -> Result<HttpResponse, Error> {
    retrieve_response(&info.url, send_request, 3).await
}

fn send_request(url: &str) -> SendClientRequest {
    info!("{}", url);
    let client = Client::default();
    client
        .get(url)
        .header("User-Agent", "rssss")
        .timeout(Duration::from_secs(60))
        .send()
}

async fn retrieve_response(
    url: &str,
    f: fn(&str) -> SendClientRequest,
    redirect_limit: u8,
) -> Result<HttpResponse, Error> {
    let mut res = f(url).await?;
    let mut counter = 0;
    loop {
        if res.status().is_success() {
            let b = res.body().limit(1_048_576).await?;
            return match rss::parse_rss(b) {
                Ok(r) => Ok(HttpResponse::Ok().json(r)),
                Err(e) => Ok(e.into()),
            };
        }
        if res.status().is_redirection() {
            if counter > redirect_limit {
                return Ok(HttpResponse::InternalServerError().finish());
            }
            let location = res.headers().get("location").and_then(|l| l.to_str().ok());
            match location {
                Some(url) => {
                    counter += 1;
                    res = f(url).await?;
                    continue;
                }
                None => return Ok(HttpResponse::InternalServerError().finish()),
            }
        };
        return Ok(HttpResponse::build(res.status()).finish());
    }
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    simple_logger::init_with_level(log::Level::Info)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut listenfd = ListenFd::from_env();

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(Cors::new().finish())
            .service(web::resource("/feed").route(web::get().to(get_feed)))
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0)? {
        server.listen(l)?
    } else {
        let host = env::var("RSSSS_BACKEND_HOST").unwrap_or("localhost".to_string());
        let port = env::var("RSSSS_BACKEND_PORT")
            .unwrap_or("8080".to_string())
            .parse::<u32>()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        server.bind(format!("{}:{}", host, port))?
    };

    server.run().await?;

    Ok(())
}
