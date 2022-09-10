async fn handle(req: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
    use futures::TryStreamExt as _;
    let mut response = hyper::Response::new(hyper::Body::empty());
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            *response.body_mut() = hyper::Body::from("Body from `/`");
        }
        (&hyper::Method::POST, "/echo/uppercase/") => {
                let mapping = req.into_body().map_ok(|chunk| {
                    chunk.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Vec<_>>()
                });
            *response.body_mut() = hyper::Body::wrap_stream(mapping);
        }
        (&hyper::Method::POST, "/echo/reverse/") => {

            let full_body = hyper::body::to_bytes(req.into_body()).await?;
            let reversed = full_body.iter().rev().cloned().collect::<Vec<u8>>();
            *response.body_mut() = reversed.into();
        }
        (&hyper::Method::POST, "/echo/") => {
            *response.body_mut() = req.into_body();
        }
        _ => {
            *response.status_mut() = hyper::StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}



#[tokio::main]
async fn main() {
    // TODO: read port from env or cli --port
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    let make_svc = hyper::service::make_service_fn(|_conn| async {
        Ok::<_, std::convert::Infallible>(hyper::service::service_fn(handle))
    });
    let server = hyper::Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
