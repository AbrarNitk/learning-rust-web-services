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


// Proxy Method
pub async fn get_out(req: actix_web::HttpRequest, data: actix_web::web::Data<OutUrl>) -> impl Responder {
    let mut headermap = reqwest::header::HeaderMap::new();
    for (key, val) in req.headers() {
        headermap.insert(key.clone(), val.clone());
    }
    let path = &req.uri().to_string()[1..];
    if path.is_empty() {
        println!("I am empty");
    }
    println!("Request URI: {}, PATH: {}, Query String: {}", req.uri(), path, req.query_string());
    "".to_string()
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
        url: reqwest::Url::parse(&format!("http://localhost:{}", CLIENTPORT)).unwrap(),
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

#[derive(Clone)]
pub struct OutUrl {
    pub url: reqwest::Url,
    pub out_client: reqwest::Client
}

#[derive(Clone)]
struct InClient {
    in_client: reqwest::Client
}