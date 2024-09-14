use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use parser::{Date, WeatherData};

/// Returns empty body
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

/// Returns a body with chunk
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn handle_req(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.method(){
        &hyper::Method::GET => {
            let body = Bytes::from("Hello, World!");
            Ok(Response::new(Full::new(body)))
        }
        _ => {
            let body = Bytes::from("Method not allowed");
            let mut res = Response::new(Full::new(body));
            *res.status_mut() = hyper::StatusCode::METHOD_NOT_ALLOWED;
            Ok(res)
        }
    }
    
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await?;

    loop{
        let (socket, _) = listener.accept().await?;
        let io = TokioIo::new(socket);

        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handle_req))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });


    }

}