use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::any::{Any, TypeId};

use http_body_util::{Full, Empty};
use hyper::body::{Body, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};

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
            if !path.starts_with("/q"){
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full("Error: path does not exist"))
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to build response: {}", e);
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(empty())
                            .unwrap()
                    }));
            }
            let query = match uri.query(){
                Some(query) => query,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Error: query required"))
                        .unwrap());
                }
            };
            let query_parts: Vec<&str> = query.split('&').collect();
            let mut query_map: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
            
            for part in query_parts{
                let kv: Vec<&str> = part.split('=').collect();
                query_map.insert(kv[0], kv[1]);
            }
            
            if query_map.len() > 2 || !query_map.contains_key("date"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Error: invalid query (only date and values allowed)"))
                    .unwrap());
            }
            else if query_map.len() == 2 && !query_map.contains_key("values"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Error: invalid query (only date and values allowed)"))
                    .unwrap());
            }
            let date_str = *query_map.get("date").unwrap();
            
            let split: Vec<&str> = date_str.split("%20").collect();
            if split.len() != 2{
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Date field must be in format YYYY-MM-DD%20YYYY-MM-DD"))
                    .unwrap());
            }
            let begin_date = match Date::from_string(split[0]){
                Ok(date) => date,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Error: invalid date format"))
                        .unwrap());
                }
            };
            let end_date = match Date::from_string(split[1]){
                Ok(date) => date,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Error: invalid date format"))
                        .unwrap());
                }
            };
            let data = data.read().unwrap();
            let map: WeatherDataMap = match data.take_range(&begin_date, &end_date){
                Some(map) => map,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(full("Error: No date found for either begin or end date"))
                        .unwrap());
                }
            };
            
            let mut points: HashSet<parser::DataPoint> = HashSet::new();
            if let Some(options) = query_map.get("values") {
                let points_str: Vec<&str> = options.split(',').collect();
                points= HashSet::with_capacity(points_str.len());
            
                for point in points_str {
                    match point {
                        "weather_code" => points.insert(parser::DataPoint::WeatherCode),
                        "temp_max" => points.insert(parser::DataPoint::TemperatureMax),
                        "temp_min" => points.insert(parser::DataPoint::TemperatureMin),
                        "precip_sum" => points.insert(parser::DataPoint::PrecipitationSum),
                        "max_wind" => points.insert(parser::DataPoint::WindSpeedMax),
                        "prob_precip_max" => points.insert(parser::DataPoint::PrecipitationProbabilityMax),
                        _ => {
                            return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(full("Error: invalid value field (check docs for valid values)"))
                                .unwrap())
                        }
                    };
                }
            }
            let json = map.json(points);
            let body = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header("Content-Length", format!("{}", json.len()))
                .body(full(json))
                .unwrap();
            return Ok(body);
        }
        &Method::POST => {
            let uri = req.uri();
            if uri.path() != "/" || uri.query() != None {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full("Error: path should be empty, no queries accepted"))
                    .unwrap());
            }
            if req.headers().get("content-type") != Some(&"application/json".parse().unwrap()){
                return Ok(Response::builder()
                    .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                    .body(full("Error: content-type must be application/json, content-type header REQUIRED"))
                    .unwrap());
            }
            let body = match String::from_utf8(req.collect().await?.to_bytes().to_vec()){
                Ok(body) => body,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                        .body(full("Error: body must be valid utf-8 text"))
                        .unwrap());
                }
            };

            let data: Vec<Value> = match serde_json::from_str(&body){
                Ok(Value::Array(data)) => data,
                Ok(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Error: body must be a json array"))
                        .unwrap());
                }
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("Error: body must be valid json"))
                        .unwrap());
                    
                }
            };

            let points = vec!["date", "weather_code", "temperature_max", "temperature_min", "precipitation_sum", "wind_speed_max", "precipitation_probability_max"];

            for item in data{
                let mut date: Date = Date::from_string("0-0-0").unwrap();
                let mut temp_max: f64 = 0.0;
                let mut temp_min: f64 = 0.0;
                let mut precip_sum: f64 = 0.0;
                let mut wind_speed_max: f64 = 0.0;
                let mut precip_prob_max: f64 = 0.0;
                let mut weather_code: u8 = 0;

                let item_obj = match item.as_object(){
                    Some(obj) => obj,
                    None => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(full("Error: body must be a JSON array of objects"))
                            .unwrap());
                    }
                };

                for point in points.iter(){
                    match *point {
                        "date" => {
                            let date_str = match item_obj.get("date"){
                                Some(date) => date,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: date field required"))
                                        .unwrap());
                                }
                            };
                            let date_str = match date_str.as_str(){
                                Some(date) => date,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: date field must be a string"))
                                        .unwrap());
                                }
                            };
                            match Date::from_string(date_str){
                                Ok(new_date) => date = new_date,
                                Err(_) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: date field must be in format YYYY-MM-DD"))
                                        .unwrap());
                                }
                            }
                        },
                        "weather_code" => {
                            
                        },
                        "temperature_max" => {},
                        "temperature_min" => {},
                        "precipitation_sum" => {},
                        "wind_speed_max" => {},
                        "precipitation_probability_max" => {},
                        _ => {
                            return Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(full("Error: Please try again"))
                                .unwrap());
                        }

                    }
                }
            }
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full("Data successfully added"))
                .unwrap());

        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(empty())
                .unwrap());
        }
    }
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