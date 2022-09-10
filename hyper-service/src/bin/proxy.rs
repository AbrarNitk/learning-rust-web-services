

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8100));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::Http::new()
                .http1_preserve_header_case(true)
                .http1_title_case_headers(true)
                .serve_connection(stream, hyper::service::service_fn(proxy))
                // .with_upgrades() // TODO: Why do we need it?
                .await
            {
                eprintln!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}

async fn proxy(
    req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    println!("req: {:?}", req);
    let host = match req.uri().host() {
        Some(t) => t,
        None => {
            let t = req.headers().get("host").expect("uri has no host");
            t.to_str().unwrap()
        }
    };
    println!("Address: {}", host);
    let stream = tokio::net::TcpStream::connect(host).await.unwrap();
    let (mut sender, conn) = hyper::client::conn::Builder::new()
        .http1_title_case_headers(true)
        .http1_preserve_header_case(true)
        .handshake(stream)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });
    let resp = sender.send_request(req).await?;
    Ok(resp)
}
