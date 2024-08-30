use std::io::Write;

mod parser;

fn main() {
    let weather_data_parsed = parser::WeatherData::from_file_path("test.txt".to_string()).unwrap();
    let mut output = std::fs::File::create("output.txt").unwrap();
    output.write_all(format!("{:?}", weather_data_parsed).as_bytes()).unwrap();
}
