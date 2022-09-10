

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
                .with_upgrades() // TODO: Why do we need it?
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
    let host = match req.uri().host() {
        Some(t) => t,
        None => {
            let t = req.headers().get("host").expect("uri has no host");
            t.to_str().unwrap()
        }
    };

    // block sites
    for x in BLOCK.iter() {
        if host.contains(x) {
            println!("#### Returning empty response, {}", host);
            return Ok(hyper::Response::new(hyper::Body::empty()));
        }
    }

    if hyper::Method::CONNECT == req.method() {
        println!("Connect Req: {:?}", req);
        if let Some(addr) = host_addr(req.uri()) {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(upgraded) => {
                        if let Err(e) = tunnel(upgraded, addr).await {
                            eprintln!("server io error: {}", e);
                        }
                    }
                    Err(e) => eprintln!("upgrade error: {}", e)
                }
            });
            return Ok(hyper::Response::new(hyper::Body::empty()));
        } else {
            eprintln!("connect host is not socket address: {:?}", req.uri());
            let mut resp = hyper::Response::new(hyper::Body::from("Bad Request"));
            *resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
            return Ok(resp)
        }
    }

    println!("req: {:?}", req);
    let address =  {
        if host.contains(":") {
            host.to_string()
        } else {
            let port = req.uri().port_u16().unwrap_or(80);
            format!("{}:{}", host, port)
        }
    };
    println!("Address: {}", address);
    let stream = tokio::net::TcpStream::connect(address).await.unwrap();
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

fn host_addr(uri: &http::Uri) -> Option<String> {
    uri.authority().and_then(|auth| Some(auth.to_string()))
}

async fn tunnel(mut upgraded: hyper::upgrade::Upgraded, addr: String) -> std::io::Result<()> {
    println!("tunnel: connecting to: {}", addr);
    // connect to remote server
    let mut server = tokio::net::TcpStream::connect(addr).await?;
    // Proxying data
    let (from_client, from_server) = tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
    println!(
        "client wrote {} bytes and received {} bytes",
        from_client, from_server
    );

    Ok(())
}

static BLOCK: [&str; 13] = [
    "googleads.g.doubleclick.net",
    "analytics.google.com",
    "c.amazon-adsystem.com",
    "www.googleadservices.com",
    "d27xxe7juh1us6.cloudfront.net",
    "clients4.google.com",
    "cdnads.geeksforgeeks.org",
    // "media.geeksforgeeks.org",
    "fundingchoicesmessages.google.com",
    "ap.lijit.com",
    "ib.adnxs.com",
    "gum.criteo.com",
    "bidder.criteo.com",
    "prg-apac.smartadserver.com"
];


// play.google.com:443
// googleads.g.doubleclick.net
// https://c.amazon-adsystem.com
// https://accounts.google.com
// https://geeksforgeeks-d.openx.ne
// https://tlx.3lift.com
// https://prg-apac.smartadserver.com
// https://securepubads.g.doubleclick.net
/*

tunnel: connecting to: play.google.com:443
req: Request { method: CONNECT, uri: www.gstatic.com:443, version: HTTP/1.1, headers: {"host": "www.gstatic.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: www.gstatic.com:443
req: Request { method: CONNECT, uri: maxcdn.bootstrapcdn.com:443, version: HTTP/1.1, headers: {"host": "maxcdn.bootstrapcdn.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: maxcdn.bootstrapcdn.com:443
req: Request { method: CONNECT, uri: clients4.google.com:443, version: HTTP/1.1, headers: {"host": "clients4.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: clients4.google.com:443
req: Request { method: CONNECT, uri: slackb.com:443, version: HTTP/1.1, headers: {"host": "slackb.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Slack/4.26.1 Chrome/100.0.4896.127 Electron/18.1.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: slackb.com:443
req: Request { method: GET, uri: http://r3.o.lencr.org/MFEwTzBNMEswSTAHBgUrDgMCGgQUSNrJoPsr0y1P8N5o0vVntzX5s8QEFBQusxe3WFbLrlAJQOYfr52LFMLGAhIDYdKCvsIVdT2n83fafmEZhao%3D, version: HTTP/1.1, headers: {"host": "r3.o.lencr.org", "x-apple-request-uuid": "4C9A31A0-86FC-4DB4-8FA4-9F0E523EB924", "proxy-connection": "keep-alive", "accept": "", "user-agent": "com.apple.trustd/2.2", "accept-language": "en-IN,en-GB;q=0.9,en;q=0.8", "accept-encoding": "gzip, deflate", "connection": "keep-alive"}, body: Body(Empty) }
Address: r3.o.lencr.org
thread 'tokio-runtime-worker' panicked at 'called `Result::unwrap()` on an `Err` value: Error { kind: InvalidInput, message: "invalid socket address" }', hyper-service/src/bin/proxy.rs:54:61
server io error: Connection reset by peer (os error 54)
req: Request { method: CONNECT, uri: play.google.com:443, version: HTTP/1.1, headers: {"host": "play.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: play.google.com:443
req: Request { method: CONNECT, uri: accounts.google.com:443, version: HTTP/1.1, headers: {"host": "accounts.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: accounts.google.com:443
req: Request { method: CONNECT, uri: f-log-extension.grammarly.io:443, version: HTTP/1.1, headers: {"host": "f-log-extension.grammarly.io:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: f-log-extension.grammarly.io:443
req: Request { method: CONNECT, uri: f-log-extension.grammarly.io:443, version: HTTP/1.1, headers: {"host": "f-log-extension.grammarly.io:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: f-log-extension.grammarly.io:443
server io error: Socket is not connected (os error 57)
req: Request { method: CONNECT, uri: ssl.gstatic.com:443, version: HTTP/1.1, headers: {"host": "ssl.gstatic.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: ssl.gstatic.com:443
req: Request { method: CONNECT, uri: docs.google.com:443, version: HTTP/1.1, headers: {"host": "docs.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: docs.google.com:443
req: Request { method: CONNECT, uri: clients6.google.com:443, version: HTTP/1.1, headers: {"host": "clients6.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: clients6.google.com:443
req: Request { method: CONNECT, uri: clients6.google.com:443, version: HTTP/1.1, headers: {"host": "clients6.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: clients6.google.com:443
req: Request { method: CONNECT, uri: d27xxe7juh1us6.cloudfront.net:443, version: HTTP/1.1, headers: {"host": "d27xxe7juh1us6.cloudfront.net:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: d27xxe7juh1us6.cloudfront.net:443
client wrote 4216 bytes and received 8289 bytes
req: Request { method: CONNECT, uri: treatment.grammarly.com:443, version: HTTP/1.1, headers: {"host": "treatment.grammarly.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: treatment.grammarly.com:443
req: Request { method: CONNECT, uri: femetrics.grammarly.io:443, version: HTTP/1.1, headers: {"host": "femetrics.grammarly.io:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: femetrics.grammarly.io:443
req: Request { method: CONNECT, uri: play.google.com:443, version: HTTP/1.1, headers: {"host": "play.google.com", "proxy-connection": "keep-alive", "connection": "keep-alive"}, body: Body(Empty) }
tunnel: connecting to: play.google.com:443
client wrote 4139 bytes and received 6458 bytes
req: Request { method: CONNECT, uri: play.google.com:443, version: HTTP/1.1, headers: {"host": "play.google.com", "proxy-connection": "keep-alive", "connection": "keep-alive"}, body: Body(Empty) }
tunnel: connecting to: play.google.com:443
client wrote 1877 bytes and received 7127 bytes
req: Request { method: CONNECT, uri: d27xxe7juh1us6.cloudfront.net:443, version: HTTP/1.1, headers: {"host": "d27xxe7juh1us6.cloudfront.net", "proxy-connection": "keep-alive", "connection": "keep-alive"}, body: Body(Empty) }
tunnel: connecting to: d27xxe7juh1us6.cloudfront.net:443
req: Request { method: CONNECT, uri: femetrics.grammarly.io:443, version: HTTP/1.1, headers: {"host": "femetrics.grammarly.io", "proxy-connection": "keep-alive", "connection": "keep-alive"}, body: Body(Empty) }
tunnel: connecting to: femetrics.grammarly.io:443
client wrote 2777 bytes and received 673 bytes
*/

/*

req: Request { method: CONNECT, uri: rtb-csync.smartadserver.com:443, version: HTTP/1.1, headers: {"host": "rtb-csync.smartadserver.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: rtb-csync.smartadserver.com:443
client wrote 8086 bytes and received 12222 bytes
client wrote 2210 bytes and received 5318 bytes
client wrote 1386 bytes and received 4457 bytes
req: Request { method: CONNECT, uri: match.adsrvr.org:443, version: HTTP/1.1, headers: {"host": "match.adsrvr.org:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: match.adsrvr.org:443
client wrote 1406 bytes and received 50906 bytes
client wrote 2150 bytes and received 1659 bytes
client wrote 8683 bytes and received 2816 bytes
client wrote 10263 bytes and received 8214 bytes
client wrote 2516 bytes and received 33364 bytes
client wrote 4783 bytes and received 69167 bytes
client wrote 1731 bytes and received 13367 bytes
client wrote 1503 bytes and received 30719 bytes
client wrote 2817 bytes and received 6331 bytes
client wrote 3243 bytes and received 2159 bytes
client wrote 3124 bytes and received 10543 bytes
client wrote 1406 bytes and received 7051 bytes
client wrote 4959 bytes and received 65371 bytes
client wrote 1618 bytes and received 79116 bytes
req: Request { method: CONNECT, uri: s.amazon-adsystem.com:443, version: HTTP/1.1, headers: {"host": "s.amazon-adsystem.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: s.amazon-adsystem.com:443
client wrote 3154 bytes and received 1853 bytes
client wrote 4784 bytes and received 3298 bytes
client wrote 3535 bytes and received 4989 bytes
client wrote 5695 bytes and received 3916 bytes
client wrote 20137 bytes and received 776 bytes
client wrote 5957 bytes and received 15641 bytes
client wrote 7505 bytes and received 42274 bytes
client wrote 8954 bytes and received 8989 bytes
client wrote 5285 bytes and received 17429 bytes
client wrote 2329 bytes and received 6393 bytes
client wrote 2941 bytes and received 6189 bytes
req: Request { method: GET, uri: http://r3.o.lencr.org/MFEwTzBNMEswSTAHBgUrDgMCGgQUSNrJoPsr0y1P8N5o0vVntzX5s8QEFBQusxe3WFbLrlAJQOYfr52LFMLGAhIEIOHnGT4ZbnE7H6VpRJu8898%3D, version: HTTP/1.1, headers: {"host": "r3.o.lencr.org", "x-apple-request-uuid": "A5CEA123-F879-4C8C-8C03-264CD34FD5A3", "proxy-connection": "keep-alive", "accept": "", "user-agent": "com.apple.trustd/2.2", "accept-language": "en-IN,en-GB;q=0.9,en;q=0.8", "accept-encoding": "gzip, deflate", "connection": "keep-alive"}, body: Body(Empty) }
Address: r3.o.lencr.org:80
client wrote 5918 bytes and received 17891 bytes
req: Request { method: CONNECT, uri: csi.gstatic.com:443, version: HTTP/1.1, headers: {"host": "csi.gstatic.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: csi.gstatic.com:443
req: Request { method: CONNECT, uri: googleads.g.doubleclick.net:443, version: HTTP/1.1, headers: {"host": "googleads.g.doubleclick.net:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: googleads.g.doubleclick.net:443
req: Request { method: CONNECT, uri: tpsc-sgc.doubleverify.com:443, version: HTTP/1.1, headers: {"host": "tpsc-sgc.doubleverify.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: tpsc-sgc.doubleverify.com:443
req: Request { method: CONNECT, uri: clients4.google.com:443, version: HTTP/1.1, headers: {"host": "clients4.google.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: clients4.google.com:443
server io error: Connection reset by peer (os error 54)
server io error: Connection reset by peer (os error 54)
client wrote 1585 bytes and received 5054 bytes
req: Request { method: CONNECT, uri: www.youtube.com:443, version: HTTP/1.1, headers: {"host": "www.youtube.com:443", "proxy-connection": "keep-alive", "user-agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36"}, body: Body(Empty) }
tunnel: connecting to: www.youtube.com:443
 */