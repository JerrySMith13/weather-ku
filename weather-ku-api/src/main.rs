use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::IndexMut;
use std::sync::{Arc, RwLock};

use http_body_util::{Full, Empty};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper::body::Frame;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};

use indexmap::IndexMap;

use parser::{DataOps, Date, WeatherData, WeatherDataMap};

fn startup() -> Arc<RwLock<WeatherDataMap>>{
    println!("Starting weather-ku-api server from specified file path");
    let args = std::env::args();
    let file_path = args.skip(1).next().expect("Error: No file path in arguments");
    println!("File path: {}", file_path);
    let file_str = std::fs::read_to_string(file_path).expect("Error: Failed to read file");
    let data = WeatherData::from_data(file_str).expect("Error: Failed to parse data (check file for errors)");
    println!("Data loaded successfully!");
    return Arc::new(RwLock::new(data));
}

async fn handle_req(req: Request<hyper::body::Incoming>, data: Arc<RwLock<WeatherDataMap>>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let method = req.method();
    let uri = req.uri();
    match method {
        &Method::GET => {
            let path = uri.path();
            if !path.starts_with("/q?"){
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap());
            }
            let query = uri.query().unwrap();
            let query_parts: Vec<&str> = query.split('&').collect();
            let mut query_map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
            
            for part in query_parts{
                let kv: Vec<&str> = part.split('=').collect();
                query_map.insert(kv[0], kv[1]);
            }
            
            if query_map.len() > 2 || !query_map.contains_key("date"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(empty())
                    .unwrap());
            }
            else if query_map.len() == 2 && !query_map.contains_key("values"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(empty())
                    .unwrap());
            }
            let date_str = *query_map.get("date").unwrap();
            
            let split: Vec<&str> = date_str.split("%20").collect();
            if split.len() != 2{
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(empty())
                    .unwrap());
            }
            let begin_date = match Date::from_string(split[0]){
                Ok(date) => date,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(empty())
                        .unwrap());
                }
            };
            let end_date = match Date::from_string(split[1]){
                Ok(date) => date,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(empty())
                        .unwrap());
                }
            };
            let data = data.read().unwrap();
            let map: WeatherDataMap = match data.take_range(&begin_date, &end_date){
                Some(map) => map,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(empty())
                        .unwrap());
                }
            };
            
            let mut points: Vec<parser::DataPoint> = Vec::new();
            if let Some(options) = query_map.get("values"){

                let points_str: Vec<&str> = options.split(',').collect();
                points = Vec::with_capacity(points_str.len());
                
                for point in points_str{
                    match point{
                        "weather_code" => points.push(parser::DataPoint::WeatherCode),
                        "temp_max" => points.push(parser::DataPoint::TemperatureMax),
                        "temp_min" => points.push(parser::DataPoint::TemperatureMin),
                        "precip_sum" => points.push(parser::DataPoint::PrecipitationSum),
                        "max_wind" => points.push(parser::DataPoint::WindSpeedMax),
                        "prob_precip_max" => points.push(parser::DataPoint::PrecipitationProbabilityMax),
                        _ => return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(empty())
                            .unwrap())
                    }
                }
            }
            
            let mut body = Response::new(full(map.json(points)));
            *body.status_mut() = StatusCode::OK;
            return Ok(body);
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(empty())
                .unwrap());
        }
    }


    Ok(Response::new(full("Hello, World!")))
}

fn full<T: Into<Bytes>>(buf: T) -> BoxBody<Bytes, hyper::Error>{
    Full::new(buf.into()).map_err(|never| match never{}).boxed()
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
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
                .serve_connection(io, service_fn(move |req| handle_req(req, data_ref.clone())))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });


    }

}