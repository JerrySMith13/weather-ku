use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper::body::Frame;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};

use parser::{Date, WeatherData, WeatherDataMap};

fn startup() -> Arc<Mutex<WeatherDataMap>>{
    println!("Starting weather-ku-api server from specified file path");
    let args = std::env::args();
    let file_path = args.skip(1).next().expect("Please provide a file path");
    let file_str = std::fs::read_to_string(file_path).expect("Failed to read file");
    let data = WeatherData::from_data(file_str).expect("Failed to parse data (check file for errors)");
    println!("Data loaded successfully!");
    return Arc::new(Mutex::new(data));
}

async fn handle_req(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    
    match req.method() {
        &Method::GET => {
        }
        _ => {
            
        }
    }
    
}

fn full<T: Into<Bytes>>(buf: T) -> BoxBody<Bytes, hyper::Error>{
    Full::new(buf.into()).map_err(|never| match never{}).boxed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    let data = startup();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await?;

    loop{
        let data_ref = data.clone();
        let (socket, _) = listener.accept().await?;
        let io = TokioIo::new(socket);
        println!("Accepted connection from: {}", io.inner().peer_addr()?);
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