use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{Read, Write, Seek, SeekFrom};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};


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

use parser::{DataPoint, DataOps, Date, WeatherData, WeatherDataMap};

#[inline]
fn add_data(point: DataPoint, points: &WeatherDataMap) -> String{
    let mut data = String::new();
    match point{
        DataPoint::WeatherCode => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.weather_code));
            }
            return data;
        },
        DataPoint::TemperatureMax => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.temp_max));
            }
            return data;
        },
        DataPoint::TemperatureMin => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.temp_min));
            }
            return data;
        },
        DataPoint::PrecipitationSum => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.precip_sum));
            }
            return data;
        },
        DataPoint::WindSpeedMax => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.max_wind));
            }
            return data;
        },
        DataPoint::PrecipitationProbabilityMax => {
            for (_, weather_data) in points.iter(){
                data.push_str(&format!(" {}",weather_data.precip_prob_max));
            }
            return data;
        }
        DataPoint::Date => {
            for (weather_data, _) in points.iter(){
                data.push_str(&format!(" {}", weather_data.to_string()));
            }
            return data;
        }


    }
}

/// REQUIRES NO DUPLICATE DATES
fn sync_file(new_data: WeatherDataMap){
    let args = std::env::args();
    let path = args.skip(1).next().expect("Error: No file path in arguments");
    let mut position = 0;
    
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .open(path){
            Ok(file) => file,
            Err(_) => {
                panic!("Error: Failed to open file");
            }
        };
    
    let mut file_str = String::new();
    match file.read_to_string(&mut file_str){
        Ok(_) => {},
        Err(_) => {
            panic!("Error: Failed to read file (file must contain ONLY valid UTF-8 text)");
        }
    };
    let lines = file_str.split('\n');
    for line in lines{
        if line.trim() == ""{
            continue;
        }
        let split: Vec<&str> = line.split(':').collect();
        if split.len() != 2{
            panic!("Error: Invalid file format");
        }
        position += split[0].len() + split[1].len() + 1;
        match file.seek(SeekFrom::Start(position as u64)){
            Ok(_) => {},
            Err(_) => panic!("Error writing to file"),
        };
        match split[0]{
            "date" => {
                match file.write_all(add_data(DataPoint::Date, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "weather_code" => {
                match file.write_all(add_data(DataPoint::WeatherCode, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "temperature_max" => {
                match file.write_all(add_data(DataPoint::TemperatureMax, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "temperature_min" => {
                match file.write_all(add_data(DataPoint::TemperatureMin, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "precipitation_sum" => {
                match file.write_all(add_data(DataPoint::PrecipitationSum, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "wind_speed_max" => {
                match file.write_all(add_data(DataPoint::WindSpeedMax, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            "precipitation_probability_max" => {
                match file.write_all(add_data(DataPoint::PrecipitationProbabilityMax, &new_data).as_bytes()){
                    Ok(_) => {},
                    Err(_) => {
                        panic!("Error: Failed to write to file");
                    }
                };
            },
            _ => {
                panic!("Error: Invalid file format");
            }

        }
    }

    /*
    Find out which line of data it is
    go to end of line
    write new data
     */
    


    
       
}

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
                                    .body(full("Error: date already exists"))
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
            drop(data_write);
            sync_file(to_add);

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