use std::collections::HashMap;

#[derive(Debug)]
/// Represents an error that can occur during parsing
pub enum ParseError{
    InvalidDate(String),
    InvalidWeatherCode(String),
    InvalidTemperature(String),
    InvalidPrecipitation(String),
    InvalidWind(String),
    InvalidPrecipitationProbability(String),
    InvalidLine(String),
    TooManyValues,
    DuplicateDate(Date),
}

type ParseResult<T> = Result<T, ParseError>;
type WeatherDataMap = HashMap<Date, WeatherData>;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
/// Struct representing a date 
pub struct Date{
    year: u32,
    month: u8,
    day: u8,
}

impl Date{
    /// Creates a new Date object from string formatted as "YYYY-MM-DD"
    pub fn from_string(date: &str) -> ParseResult<Date>{
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() != 3{
            return Err(ParseError::InvalidDate(format!("Invalid date: {}", date))); 
        }
        let year = match parts[0].parse(){
            Ok(year) => year,
            Err(_) => return Err(ParseError::InvalidDate(format!("Invalid year: {}", parts[0]))),
        };
        let month = match parts[1].parse(){
            Ok(month) => month,
            Err(_) => return Err(ParseError::InvalidDate(format!("Invalid month: {}", parts[1]))),
        }; //FIXME: Unwrap
        let day = match parts[2].parse(){
            Ok(day) => day,
            Err(_) => return Err(ParseError::InvalidDate(format!("Invalid day: {}", parts[2]))),
        };
        Ok(Date{
            year,
            month,
            day,
        })
    }
    pub fn to_string(&self) -> String{
        format!("{}-{}-{}", self.year, self.month, self.day)
    }
}


//FIXME: MAKE THIS A MACRO PLEASEEEEEEE I NEED TO LEARN MACROS
#[inline]
fn parse_date(data: &str) -> ParseResult<Vec<Date>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut dates: Vec<Date> = Vec::with_capacity(split.len());
    for date in split{
        match Date::from_string(date){
            Ok(date) => if dates.contains(&date) {
                return Err(ParseError::DuplicateDate(date));
            } else {dates.push(date);},
            Err(e) => return Err(e),
        }
    }
    Ok(dates)
}

#[inline]
fn parse_weather_code(data: &str) -> ParseResult<Vec<u8>> {
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut codes: Vec<u8> = Vec::with_capacity(split.len());
    for code in split {
        codes.push(match code.parse::<f64>() {
            Ok(code) => {
                if code < 0.0 || code > u8::MAX as f64 {
                    return Err(ParseError::InvalidWeatherCode(code.to_string()));
                }
                code as u8 // Truncate the decimal part
            },
            Err(_) => return Err(ParseError::InvalidWeatherCode(code.to_string())),
        });
    }
    Ok(codes)
}

#[inline]
fn parse_temp_max(data: &str) -> ParseResult<Vec<f32>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut temps: Vec<f32> = Vec::with_capacity(split.len());
    for temp in split{
        temps.push(match temp.parse(){
            Ok(temp) => temp,
            Err(_) => return Err(ParseError::InvalidTemperature(temp.to_string())),
        });
    }
    Ok(temps)
}

#[inline]
fn parse_temp_min(data: &str) -> ParseResult<Vec<f32>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut temps: Vec<f32> = Vec::with_capacity(split.len());
    for temp in split{
        temps.push(match temp.parse(){
            Ok(temp) => temp,
            Err(_) => return Err(ParseError::InvalidTemperature(temp.to_string())),
        });
    }
    Ok(temps)
}

#[inline]
fn parse_precip_sum(data: &str) -> ParseResult<Vec<f32>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut sums: Vec<f32> = Vec::with_capacity(split.len());
    for sum in split{
        sums.push(match sum.parse(){
            Ok(sum) => sum,
            Err(_) => return Err(ParseError::InvalidPrecipitation(sum.to_string())),
        });
    }
    Ok(sums)
    
}

#[inline]
fn parse_wind_max(data: &str) -> ParseResult<Vec<f32>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut winds: Vec<f32> = Vec::with_capacity(split.len());
    for wind in split{
        winds.push(match wind.parse(){
            Ok(wind) => wind,
            Err(_) => return Err(ParseError::InvalidWind(wind.to_string())),
        });
    }
    Ok(winds)

}

#[inline]
fn parse_prob_max(data: &str) -> ParseResult<Vec<f32>>{
    let split: Vec<&str> = data.split_whitespace().collect();
    let mut probs: Vec<f32> = Vec::with_capacity(split.len());
    for prob in split{
        probs.push(match prob.parse(){
            Ok(prob) => prob,
            Err(_) => return Err(ParseError::InvalidPrecipitationProbability(prob.to_string())),
        });
    }
    Ok(probs)
}

#[derive(Debug, Clone)]
/// Struct representing a single weather data entry
pub struct WeatherData{
    pub date: Date,
    pub weather_code: u8,
    pub temp_max: f32,
    pub temp_min: f32,
    pub precip_sum: f32,
    pub max_wind: f32,
    pub precip_prob_max: f32,
}
impl WeatherData{
    
    /// Creates a new WeatherData object from given parameters
    pub fn new(date: Date, weather_code: u8, temp_max: f32, temp_min: f32, precip_sum: f32, max_wind: f32, precip_prob_max: f32) -> WeatherData{
        WeatherData{
            date,
            weather_code,
            temp_max,
            temp_min,
            precip_sum,
            max_wind,
            precip_prob_max,
        }
    }
    
    
    pub fn from_data(data: String) -> ParseResult<WeatherDataMap>{

        let mut dates: Vec<Date> = vec![];
        let mut weather_codes: Vec<u8> = vec![];
        let mut temp_maxs: Vec<f32> = vec![];
        let mut temp_mins: Vec<f32> = vec![];
        let mut precip_sums: Vec<f32> = vec![];
        let mut wind_maxs: Vec<f32> = vec![];
        let mut prob_maxs: Vec<f32> = vec![];
        
        /*
        split by colon, then split by comma
         */

        let mut line_count: u8 = 0;
        let mut lines = data.lines();
        while let Some(line) = lines.next(){
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() != 2{
                return Err(ParseError::InvalidLine(line.to_string()));
            }
            match parts[0].trim(){
                "date" => dates = match parse_date(parts[1]){
                    Ok(dates) => dates,
                    Err(e) => return Err(e),
                },
                "weather_code" => weather_codes = match parse_weather_code(parts[1]){
                    Ok(codes) => codes,
                    Err(e) => return Err(e),
                },
                "temperature_max" => temp_maxs = match parse_temp_max(parts[1]){
                    Ok(temps) => temps,
                    Err(e) => return Err(e),
                },
                "temperature_min" => temp_mins = match parse_temp_min(parts[1]){
                    Ok(temps) => temps,
                    Err(e) => return Err(e),
                },
                "precipitation_sum" => precip_sums = match parse_precip_sum(parts[1]){
                    Ok(sums) => sums,
                    Err(e) => return Err(e),
                },
                "wind_speed_max" => wind_maxs = match parse_wind_max(parts[1]){
                    Ok(winds) => winds,
                    Err(e) => return Err(e),
                },
                "precipitation_probability_max" => prob_maxs = match parse_prob_max(parts[1]){
                    Ok(probs) => probs,
                    Err(e) => return Err(e),
                },
                _ => return Err(ParseError::InvalidLine(line.to_string())),

            }

            line_count += 1;
        }

        if line_count != 7 {
            return Err(ParseError::InvalidLine("Invalid number of lines".to_string()));
        }

        let mut i: usize = 0;
        let mut weather_data: Vec<WeatherData> = Vec::with_capacity(dates.len());
        while let Some(date) = dates.get(i){
            let weather_code = match weather_codes.get(i){
                Some(code) => code,
                None => return Err(ParseError::TooManyValues),
            };
            let temp_max = match temp_maxs.get(i){
                Some(temp) => temp,
                None => return Err(ParseError::TooManyValues),
            };
            let temp_min = match temp_mins.get(i){
                Some(temp) => temp,
                None => return Err(ParseError::TooManyValues),
            };
            let precip_sum = match precip_sums.get(i){
                Some(sum) => sum,
                None => return Err(ParseError::TooManyValues),
            };
            let wind_max = match wind_maxs.get(i){
                Some(wind) => wind,
                None => return Err(ParseError::TooManyValues),
            };
            let precip_prob_max = match prob_maxs.get(i){
                Some(prob) => prob,
                None => return Err(ParseError::TooManyValues),
            };
            weather_data.push(WeatherData::new(*date, *weather_code, *temp_max, *temp_min, *precip_sum, *wind_max, *precip_prob_max));
            i += 1;
        }

        
        weather_data.sort_by(|a, b|comp_date(a, b));

        let mut weather_data_map: HashMap<Date, WeatherData> = HashMap::with_capacity(weather_data.len());

        for data in weather_data{
            weather_data_map.insert(data.date, data);
        }
        
        Ok(weather_data_map)
        
            



    }
}


fn comp_date(a: &WeatherData, b: &WeatherData) -> std::cmp::Ordering{
    let a = a.date;
    let b = b.date;
    if a.year == b.year{
        if a.month == b.month{
            a.day.cmp(&b.day)
        }else{
            a.month.cmp(&b.month)
        }
    }
    else{
        a.year.cmp(&b.year)
    }

}