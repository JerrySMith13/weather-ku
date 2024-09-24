use std::collections::HashSet;

use indexmap::IndexMap;

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

#[derive(PartialEq, Eq, Hash)]
pub enum DataPoint{
    WeatherCode,
    TemperatureMax,
    TemperatureMin,
    PrecipitationSum,
    WindSpeedMax,
    PrecipitationProbabilityMax,
    Date
}

pub type ParseResult<T> = Result<T, ParseError>;
pub type WeatherDataMap = IndexMap<Date, WeatherData>;

pub trait DataOps{
    fn take_range(&self, begin: &Date, end: &Date) -> Option<WeatherDataMap>;
    fn json(self, points: HashSet<DataPoint>) -> String;
}

impl DataOps for WeatherDataMap{
    fn take_range(&self, begin: &Date, end: &Date) -> Option<WeatherDataMap>{
        let begin_index = match self.get_index_of(begin){
            Some(index) => index,
            None => return None,
        };
        let end_index = match self.get_index_of(end){
            Some(index) => index,
            None => return None,
        };

        let mut map: WeatherDataMap = IndexMap::with_capacity(begin_index.abs_diff(end_index) as usize);

        for (date, data) in self.iter(){
            if date >= begin && date <= end{
                map.insert(*date, data.clone());
            }
        }
        Some(map)
    }
    fn json(self, mut options: HashSet<DataPoint>) -> String{
        let mut json = String::from("[");
        for (_, data) in self.iter(){
            json.push_str(&data.json(&mut options));
            json.push_str(",");
        }
        json.remove(json.rfind(',').unwrap());
        json.push_str("]");
        json
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq, PartialOrd, Ord)]
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
    pub fn distance(begin: &Date, end: &Date) -> u32{
        let tot_days_1 = begin.year * 365 + begin.month as u32 * 30 + begin.day as u32;
        let tot_days_2 = end.year * 365 + end.month as u32 * 30 + end.day as u32;
        tot_days_2.abs_diff(tot_days_1)
        
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

        let mut weather_data_map: IndexMap<Date, WeatherData> = IndexMap::with_capacity(weather_data.len());

        for data in weather_data{
            weather_data_map.insert(data.date, data);
        }
        
        Ok(weather_data_map)
    }

    fn json(&self, points: &mut HashSet<DataPoint>) -> String{
        if points.is_empty(){
            *points = vec![DataPoint::WeatherCode, DataPoint::TemperatureMax, DataPoint::TemperatureMin, DataPoint::PrecipitationSum, DataPoint::WindSpeedMax, DataPoint::PrecipitationProbabilityMax].into_iter().collect();
        }
        let mut json = String::from("{");
        json.push_str(&format!("\"date\":\"{}\",", self.date.to_string()));
        for point in points.iter(){
            match point{
                DataPoint::WeatherCode => json.push_str(&format!("\"weather_code\":{},", self.weather_code)),
                DataPoint::TemperatureMax => json.push_str(&format!("\"temperature_max\":{},", self.temp_max)),
                DataPoint::TemperatureMin => json.push_str(&format!("\"temperature_min\":{},", self.temp_min)),
                DataPoint::PrecipitationSum => json.push_str(&format!("\"precipitation_sum\":{},", self.precip_sum)),
                DataPoint::WindSpeedMax => json.push_str(&format!("\"wind_speed_max\":{},", self.max_wind)),
                DataPoint::PrecipitationProbabilityMax => json.push_str(&format!("\"precipitation_probability_max\":{},", self.precip_prob_max)),
                DataPoint::Date => (),
                
            }
        }
        // Removes trailing comma
        json.remove(json.rfind(',').unwrap());
        json.push_str("}");
        json
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