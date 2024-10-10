use std::collections::HashSet;
use std::fs::OpenOptions;
use std::time::Duration;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock, Condvar, Mutex};
use std::io::Write;


use http_body_util::{Full, Empty};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use indexmap::IndexMap;
use serde_json::Value;
use tokio::net::TcpListener;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};

use chrono::DurationRound;

use parser::{DataOps, Date, WeatherData, WeatherDataMap};

fn log(msg: &str){
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.txt"){
            Ok(f) => f,
            Err(e) => {
                match e.kind(){
                    std::io::ErrorKind::PermissionDenied => {
                        eprintln!("Error writing to log file: Permission Denied");
                        std::process::exit(1);
                    },
                    _ => {
                        eprintln!("Error with log file: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        };
        
    let mut log_msg = String::new();
    log_msg.push('\n');
    let date_time = chrono::Utc::now().duration_round(chrono::TimeDelta::try_milliseconds(10).unwrap()).unwrap().to_string();
    log_msg.push_str(&date_time);
    log_msg.push_str(": ");
    log_msg.push_str(msg);
    match file.write_all(log_msg.as_bytes()){
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error writing to log file: {:?}", e);
            std::process::exit(1);
        }
    };
    print!("{}", log_msg);
    
}

async fn heartbeat(data: Arc<RwLock<WeatherDataMap>>, quit: Arc<Mutex<bool>>){

    let path = std::env::args().skip(1).next().unwrap();
    
    log("Started heartbeat process");
    loop{
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {},
            _ = tokio::signal::ctrl_c() => {
                *quit.lock().unwrap() = true;
            }
        }
        let mut file = match OpenOptions::new().write(true).truncate(true).open(path.clone()){
            Ok(file) => file,
            Err(e) => panic!("Error: {}", e),
        };

        match file.write_all(data.read().unwrap().to_file().as_bytes()){
            Ok(_) => {},
            Err(e) => panic!("File error: {}", e)
        };
        log("Server updated by heartbeat thread");
        if *quit.lock().unwrap() {
            break;
        }

    }
    
    
}

fn startup() -> Arc<RwLock<WeatherDataMap>>{
    log("Starting weather-ku-api server from specified file path");
    let args = std::env::args();
    let file_path = args.skip(1).next().expect("Error: No file path in arguments");
    let file_str = std::fs::read_to_string(&file_path).expect("Error: could not read from specified file path");
    let data = WeatherData::from_data(file_str).expect("Error: Failed to parse data (check file for errors)");
    log("Data loaded successfully!");
    let data = Arc::new(RwLock::new(data));
    data
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
                    log(format!("Failed to build response: {:?}", e).as_str());
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
            
            if query_map.len() > 2 || !query_map.contains_key("dates"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Error: invalid query (only dates and values allowed)"))
                    .unwrap());
            }
            else if query_map.len() == 2 && !query_map.contains_key("values"){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Error: invalid query (only dates and values allowed)"))
                    .unwrap());
            }
            let date_str = *query_map.get("dates").unwrap();
            
            let split: Vec<&str> = date_str.split("%20").collect();
            if split.len() != 2{
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("Dates field must be in format YYYY-MM-DD%20YYYY-MM-DD"))
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
        },
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

            let values: Vec<Value> = match serde_json::from_str(&body){
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
            let mut to_add: WeatherDataMap = IndexMap::with_capacity(values.len());
            for item in values{
                let mut date: Date = Date::from_string("0-0-0").unwrap();
                let mut temp_max: f32 = 0.0;
                let mut temp_min: f32 = 0.0;
                let mut precip_sum: f32 = 0.0;
                let mut wind_speed_max: f32 = 0.0;
                let mut precip_prob_max: f32 = 0.0;
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
                            let new_date = match Date::from_string(date_str){
                                Ok(new_date) => new_date,
                                Err(_) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: date field must be in format YYYY-MM-DD"))
                                        .unwrap());
                                }
                            };
                            if data.read().unwrap().contains_key(&new_date){
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{ \"error\": \"Date already exists\" }"))
                                    .unwrap());
                            }
                            date = new_date;
                        },
                        "weather_code" => {
                            let code = match item_obj.get("weather_code"){
                                Some(code) => code,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: weather_code field required"))
                                        .unwrap());
                                }
                            };
                            let code = match code.as_u64() {
                                Some(code) => if code > 255 {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: weather_code field must be a number between 0 and 100"))
                                        .unwrap());
                                } else {
                                    code
                                },
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: weather_code field must be a number"))
                                        .unwrap());
                                }
                            };
                            weather_code = code as u8;
                        },
                        "temperature_max" => {
                            let temp = match item_obj.get("temperature_max"){
                                Some(temp) => temp,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: temperature_max field required"))
                                        .unwrap());
                                }
                            };
                            let temp = match temp.as_f64(){
                                Some(temp) => temp,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: temperature_max field must be a number"))
                                        .unwrap());
                                }
                            };
                            temp_max = temp as f32;
                        },
                        "temperature_min" => {
                            let temp = match item_obj.get("temperature_min"){
                                Some(temp) => temp,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: temperature_min field required"))
                                        .unwrap());
                                }
                            };
                            let temp = match temp.as_f64(){
                                Some(temp) => temp,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: temperature_min field must be a number"))
                                        .unwrap());
                                }
                            };
                            temp_min = temp as f32;
                        },
                        "precipitation_sum" => {
                            let precip = match item_obj.get("precipitation_sum"){
                                Some(precip) => precip,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: precipitation_sum field required"))
                                        .unwrap());
                                }
                            };
                            let precip = match precip.as_f64(){
                                Some(precip) => precip,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: precipitation_sum field must be a number"))
                                        .unwrap());
                                }
                            };
                            precip_sum = precip as f32;
                        },
                        "wind_speed_max" => {
                            let wind = match item_obj.get("wind_speed_max"){
                                Some(wind) => wind,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: wind_speed_max field required"))
                                        .unwrap());
                                }
                            };
                            let wind = match wind.as_f64(){
                                Some(wind) => wind,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: wind_speed_max field must be a number"))
                                        .unwrap());
                                }
                            };
                            wind_speed_max = wind as f32;
                        },
                        "precipitation_probability_max" => {
                            let prob = match item_obj.get("precipitation_probability_max"){
                                Some(prob) => prob,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: precipitation_probability_max field required"))
                                        .unwrap());
                                }
                            };
                            let prob = match prob.as_f64(){
                                Some(prob) => prob,
                                None => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("Error: precipitation_probability_max field must be a number"))
                                        .unwrap());
                                }
                            };
                            precip_prob_max = prob as f32;
                        },
                        _ => {
                            return Ok(Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(full("Error: Please try again"))
                                .unwrap());
                        }

                    }
                }
                if to_add.contains_key(&date){
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full(format!("Error: duplicate date found: {}", date.to_string())))
                        .unwrap());
                }
                to_add.insert(date,WeatherData::new(date, weather_code, temp_max, temp_min, precip_sum, wind_speed_max, precip_prob_max));
            }

            let mut data_write = data.write().unwrap();
            for item in &to_add{
                data_write.insert(item.0.clone(), item.1.clone());

            }
            
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full("Data successfully added"))
                .unwrap());

        },
        &Method::PUT => {
            let path = uri.path();
            if !path.starts_with("/q"){
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full("{\"error\": \"path does not exist\"}"))
                    .unwrap_or_else(|e| {
                    log(format!("Failed to build response: {:?}", e).as_str());
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
                        .body(full("{\"error\": \"date query required\"}"))
                        .unwrap());
                }
            };
            if !query.starts_with("dates="){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("{\"error\": \"date query required\"}"))
                    .unwrap());
            }
            let date_str = query.strip_prefix("dates=").unwrap();
            let dates: Vec<&str> = date_str.split("%20").collect();
            let mut dates_to_change: Vec<Date> = Vec::with_capacity(dates.len());
            for date_str in dates{
                let date = match Date::from_string(date_str){
                    Ok(date) => date,
                    Err(_) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(full("{\"error\": \"invalid date format\"}"))
                            .unwrap());
                    }
                };
                dates_to_change.push(date);
            }
            
            let body = match String::from_utf8(req.collect().await?.to_bytes().to_vec()){
                Ok(body) => body,
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                        .body(full("{\"error\": \"body must be valid utf-8 text\"}"))
                        .unwrap());
                }
            };
            let values: Vec<Value> = match serde_json::from_str(&body){
                Ok(Value::Array(data)) => data,
                Ok(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("{\"error\": \"body must be a json array\"}"))
                        .unwrap());
                }
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("{\"error\": \"body must be valid json\"}"))
                        .unwrap());
                    
                }
            };
            if values.len() != dates_to_change.len(){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("{\"error\": \"number of dates and values must be equal\"}"))
                    .unwrap());
            }
            let mut index: u16 = 0;
            let mut data = data.write().unwrap();
            for value in values{
                if !value.is_object(){
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("{\"error\": \"body must be a json array of objects\"}"))
                        .unwrap());
                }
                let value = value.as_object().unwrap();
                match data.get_mut(&dates_to_change[index as usize]){
                    Some(changing) => {
                        if let Some(weather_code) = value.get("weather_code"){
                            if let Some(weather_code) = weather_code.as_u64(){
                                if weather_code > 255{
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(full("{\"error\": \"weather_code must be a number between 0 and 255\"}"))
                                        .unwrap());
                                }
                                changing.weather_code = weather_code as u8;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"weather_code must be a number\"}"))
                                    .unwrap());
                        }
                    }
                        if let Some(temp_max) = value.get("temperature_max"){
                            if let Some(temp_max) = temp_max.as_f64(){
                                changing.temp_max = temp_max as f32;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"temperature_max must be a number\"}"))
                                    .unwrap());
                            }
                        }
                        if let Some(temp_min) = value.get("temperature_min"){
                            if let Some(temp_min) = temp_min.as_f64(){
                                changing.temp_min = temp_min as f32;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"temperature_min must be a number\"}"))
                                    .unwrap());
                            }
                        }
                        if let Some(precip_sum) = value.get("precipitation_sum"){
                            if let Some(precip_sum) = precip_sum.as_f64(){
                                changing.precip_sum = precip_sum as f32;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"precipitation_sum must be a number\"}"))
                                    .unwrap());
                            }
                        }
                        if let Some(wind_speed_max) = value.get("wind_speed_max"){
                            if let Some(wind_speed_max) = wind_speed_max.as_f64(){
                                changing.max_wind = wind_speed_max as f32;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"wind_speed_max must be a number\"}"))
                                    .unwrap());
                            }
                        }
                        if let Some(precip_prob_max) = value.get("precipitation_probability_max"){
                            if let Some(precip_prob_max) = precip_prob_max.as_f64(){
                                changing.precip_prob_max = precip_prob_max as f32;
                            }
                            else{
                                return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .body(full("{\"error\": \"precipitation_probability_max must be a number\"}"))
                                    .unwrap());
                            }
                        }
                    },
                    None => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(full(format!("{{\"error\": \"date {} does not exist\"}}", dates_to_change[index as usize].to_string())))
                            .unwrap());
                    }
                }
                index += 1;
            }
            return Ok(Response::builder()
        .status(StatusCode::OK)
        .body(full("{\"success\": \"Data successfully updated\"}"))
        .unwrap());
        },      
        &Method::DELETE => {
            if uri.path() != "/q"{
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full("{\"error\": \"path does not exist\"}"))
                    .unwrap());
            }
            let query = match uri.query(){
                Some(query) => query,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full("{\"error\": \"date query required\"}"))
                        .unwrap());
                }
            };
            if !query.starts_with("dates="){
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(full("{\"error\": \"date query required\"}"))
                    .unwrap());
            }
            let date_str = query.strip_prefix("dates=").unwrap();
            let dates: Vec<&str> = date_str.split("%20").collect();
            let mut dates_to_delete: Vec<Date> = Vec::with_capacity(dates.len());
            for date_str in dates{
                let date = match Date::from_string(date_str){
                    Ok(date) => date,
                    Err(_) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(full(format!("{{\"error\": \"invalid date format: {}\"}}", date_str)))
                            .unwrap());
                    }
                };
                dates_to_delete.push(date);
            }
            let mut data = data.write().unwrap();
            for date in dates_to_delete{
                if !data.contains_key(&date){
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(full(format!("{{\"error\": \"date {} does not exist\"}}", date.to_string())))
                        .unwrap());
                }
                data.shift_remove_full(&date);
            }
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full("{\"success\": \"Data successfully deleted\"}"))
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

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    let data = startup();
    let is_quit = Arc::new(Mutex::new(false));
    let heartbeat_thread = tokio::spawn(heartbeat(data.clone(), is_quit.clone()));
    data.clear_poison();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(&addr).await?;
    let http = http1::Builder::new();
    let graceful = hyper_util::server::graceful::GracefulShutdown::new();
    let mut signal = std::pin::pin!(shutdown_signal());
    
    loop{
        tokio::select! {
            Ok((stream, _addr)) = listener.accept() => {
                let io = TokioIo::new(stream);
                let data_ref = data.clone();
                let conn = http.serve_connection(io, service_fn(move |req| {handle_req(req, data_ref.clone())}));
                // watch this connection
                let fut = graceful.watch(conn);
                tokio::spawn(async move {
                    if let Err(e) = fut.await {
                        log(format!("Error serving connection: {:?}", e).as_str());
                    }
                });
            },
    
            _ = &mut signal => {
                log("Graceful shutdown signal received");
                // stop the accept loop
                break;
            }
        }
    }
    tokio::select! {
        // Waits for all connections to close, then waits for the heartbeat thread to finish updating file
        _ = graceful.shutdown() => {
            *is_quit.lock().unwrap() = true;
            match heartbeat_thread.await{
                Ok(_) => {},
                Err(e) => {
                    log(format!("Error with shutting down heartbeat thread: {:?}", e).as_str());
                }
            };
            log("Server shutdown completed without errors");

        },
        // If the graceful shutdown times out, print an error message
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            *is_quit.lock().unwrap() = true;
            match heartbeat_thread.await{
                Ok(_) => {},
                Err(e) => {
                    log(format!("Error with shutting down heartbeat thread: {:?}", e).as_str());
                }
            };
            log("Server timed out wait for all connections to close");
        }
    }
    Ok(())

}