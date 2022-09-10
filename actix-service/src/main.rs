use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod utils;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn handle_response(response: reqwest::Response) -> actix_web::HttpResponse {
    dbg!(&response.content_length());
    // dbg!(&response.text().await.unwrap());

    // let len = match response.content_length() {
    //     Some(l) => l as usize,
    //     None => {
    //         return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
    //             HotMess::InvalidLength,
    //         ));
    //     }
    // };
    //

    let content = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(
                HotMess::InvalidBody(e.to_string()),
            ))
        }
    };

    let mut w = std::io::BufWriter::new(vec![0u8; content.len()]);
    let mut r = std::io::BufReader::new(&content[..]);

    match std::io::copy(&mut r, &mut w) {
        Ok(n) if n == 0 => return actix_web::HttpResponse::Ok().body(""),
        Ok(_) => {}
        Err(e) => {
            return actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(HotMess::IoCopyErr(
                e.to_string(),
            )))
        }
    }

    match w.into_inner() {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(e) => actix_web::HttpResponse::from(actix_web::error::ErrorInternalServerError(HotMess::FailedBufFlush(
            e.to_string(),
        ))),
    }

    // unimplemented!()
}

// Proxy Method
pub async fn get_out(
    req: actix_web::HttpRequest,
    data: actix_web::web::Data<OutUrl>,
) -> impl Responder {
    let mut headermap = reqwest::header::HeaderMap::new();
    for (key, val) in req.headers() {
        headermap.insert(key.clone(), val.clone());
    }
    // Query Parameter are the parts of it
    let path = &req.uri().to_string()[1..];
    if path.is_empty() {
        println!("I am empty");
    }
    println!(
        "Request URI: {}, PATH: {}, Query String: {}",
        req.uri(),
        path,
        req.query_string()
    );
    let client = data.out_client.clone();
    let req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(&format!("{}/{}", data.url.as_str(), path)).unwrap());
    let handle = client.execute(req).await;
    match handle {
        Ok(res) => handle_response(res).await,
        Err(e) => HttpResponse::InternalServerError().body(format!("Error requesting path: {}", e)),
    }
}

const CLIENTPORT: &str = "32655";
const PROXYINPORT: &str = "62081"; // which port will the reverse proxy use for making outgoing request
const PROXYOUTPORT: &str = "62082"; // which port the reverse proxy listens on

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = utils::parse_from_cli("--port").unwrap_or_else(|| "8000".to_string());

    let out_client = reqwest::ClientBuilder::new()
        .http2_adaptive_window(true)
        .tcp_keepalive(std::time::Duration::new(150, 0))
        .tcp_nodelay(true) // disable Nagle
        // .connect_timeout(Duration::new(150, 0))
        .connection_verbose(true)
        .build()
        .expect("Failed creating out client pool");

    let redirect_url = OutUrl {
        url: reqwest::Url::parse(&format!("https://www.google.com/")).unwrap(),
        out_client,
    };
    println!("Redirect URL: {}", redirect_url.url);

    // let mut headers = reqwest::header::HeaderMap::new();
    // headers.insert("X-Forwarded-For", reqwest::header::HeaderValue::from_static(""));
    //
    // let in_client = reqwest::ClientBuilder::new()
    //     .http2_adaptive_window(true)
    //     .tcp_keepalive(std::time::Duration::new(150, 0))
    //     .tcp_nodelay(true) // disable Nagle
    //     .connection_verbose(true)
    //     .default_headers(headers)
    //     .build()
    //     .expect("Failed creating in client pool");
    // let in_c = InClient { in_client };

    println!("Server Started!!!");

    HttpServer::new(move || {
        App::new()
            .default_service(web::route().to(get_out))
            .data(redirect_url.clone())
    })
    .bind(format!("127.0.0.1:{}", PROXYOUTPORT))?
    .run()
    .await

    // let s_in = HttpServer::new(move || {
    //     App::new()
    //         .default_service(web::route().to(get_in))
    //         .data(in_c.clone())
    // })
    //     .bind(format!("127.0.0.1:{}", PROXYINPORT))?
    //     .run();

    // futures::future::try_join(s_in, s_out).await?;

    // Ok(())
    // HttpServer::new(|| {
    //     App::new()
    //         .service(hello)
    //         .service(echo)
    //         .route("/hey", web::get().to(manual_hello))
    // })
    // .bind(("127.0.0.1", port.parse().unwrap()))?
    // .run()
    // .await
}

#[derive(thiserror::Error, Debug)]
enum HotMess {
    #[error("Invalid length received")]
    InvalidLength,
    #[error("Could not fetch body: {0}")]
    InvalidBody(String),
    #[error("Could not copy response body: {0}")]
    IoCopyErr(String),
    #[error("Failed to flush the buffer: {0}")]
    FailedBufFlush(String),
}

#[derive(Clone)]
pub struct OutUrl {
    pub url: reqwest::Url,
    pub out_client: reqwest::Client,
}

#[derive(Clone)]
struct InClient {
    in_client: reqwest::Client,
}
