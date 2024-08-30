use std::fs::File;
use std::io::Read;
use std::io::BufReader;

/// Represents an error that can occur during parsing
enum ParseError{
    InvalidDate(String),
    InvalidWeatherCode(String),
    InvalidTemperature(String),
    InvalidPrecipitation(String),
    InvalidWind(String),
    InvalidPrecipitationProbability(String),
    InvalidFilePath(String),
}

/// Struct representing a date 
pub struct Date{
    year: u32,
    month: u8,
    day: u8,
}

impl Date{
    pub fn from_string(date: &str) -> Date{
        let mut parts = date.split('-');
        let year = parts.next().unwrap().parse().unwrap();
        let month = parts.next().unwrap().parse().unwrap(); //FIXME: Unwrap
        let day = parts.next().unwrap().parse().unwrap();
        Date{
            year,
            month,
            day,
        }
    }
}

/// Struct representing a single weather data entry
pub struct WeatherData{
    date: Date,
    weather_code: u8,
    temp_max: f32,
    temp_min: f32,
    precip_sum: f32,
    max_wind: f32,
    precip_prob_max: f32,
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
    
    
    pub fn from_file_path(path: String) -> Result<Vec<WeatherData>, ParseError>{
        let mut data = match std::fs::read_to_string(path){
            Ok(data) => data,
            Err(_) => return Err(ParseError::InvalidFilePath(path)),
        }; 
        
        
            



    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_date_from_string(){
        let date = Date::from_string("2020-01-01");
        assert_eq!(date.year, 2020);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 1);
    }
}