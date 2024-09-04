use std::{option, process};
use inquire::{Confirm, Editor, MultiSelect, Select};
use parser::{Date, WeatherData, ParseError};
mod parser;
mod pathfinder;

fn exit_dialog(menu: fn()) {
    let exit = Confirm::new("Are you sure you want to exit?").prompt();
    match exit {  
        Ok(true) => process::exit(0),
        Ok(false) => menu(),
        Err(_) => process::exit(1),
    }
}
// INPUT METHODS \\
fn data_from_file(path: String){
    let data = match std::fs::read_to_string(path){
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            start_menu();
            return;
        }
    };

    data_point_options(data.as_str());
}

fn data_from_manual(){
    let editor_data = Editor::new("Enter data here:").with_help_message("Save and close the file to submit!").prompt();
    match editor_data {
        Ok(data) => {
            data_point_options(data.as_str());
        },
        Err(_) => start_menu(),
    }

}

//END INPUT METHODS\\


fn start_menu() {
    let menu_options = vec!["Parse data from file", "Enter data manually", "Exit"];
    let select_option = Select::new("Welcome! Select an option:", menu_options)
    .prompt();

    match select_option {
        Ok(option) => {
            match option {
                "Parse data from file" => data_from_file(pathfinder::file_dialog(".")),
                "Enter data manually" => data_from_manual(),
                "Exit" => exit_dialog(start_menu),
                _ => println!("Invalid option!"),
            }
        },
        Err(_) => start_menu(),
    }
        
}

fn data_point_options(data: &str){
    let menu_option = vec!["Weather Code", "High Temperature", "Low Temperature", "Total Precipitation", "Highest Precipitation Chance", "Maximum Wind Speed", "Return to menu", "Exit"];
    let select_option = Select::new("Select an data point to sample:", menu_option).prompt();
    match select_option{
        Ok(option) => {
            match option{
                "Weather Code" => {
                    println!("Weather Code");
                },
                "High Temperature" => {
                    println!("High Temperature");
                },
                "Low Temperature" => {
                    println!("Low Temperature");
                },
                "Total Precipitation" => {
                    println!("Total Precipitation");
                },
                "Highest Precipitation Chance" => {
                    println!("Highest Precipitation Chance");
                },
                "Maximum Wind Speed" => {
                    println!("Maximum Wind Speed");
                },
                "Return to menu" => {
                    start_menu();
                },
                "Exit" => {
                    exit_dialog(start_menu);
                },
                _ => {
                    data_point_options(data);
                },
            }
        },
        Err(_) => start_menu(),
    }
}

fn date_range(data: &str) -> Vec<WeatherData>{
    let data = WeatherData::from_data(data.to_string());
    match data{
        Ok(data) => {
            let mut dates_to_display: Vec<String>= Vec::with_capacity(data.len());
            for node in data.values(){
                dates_to_display.push(node.date.to_string());
            }
            let select_options = match MultiSelect::new("Select dates to sample:", dates_to_display).prompt(){
                Ok(options) => options,
                Err(_) => {
                    println!("Sorry! Error Occured");
                    start_menu();
                    return Vec::new();
                },
            };
            let mut selected_dates: Vec<WeatherData> = Vec::with_capacity(select_options.len());
            for option in select_options{
                selected_dates.push(data.get(&parser::Date::from_string(option.as_str()).unwrap()).unwrap().clone());
            }
            selected_dates
                    
        },
        Err(err) => {handle_parse_err(err); return Vec::new();},
    }

    
}

#[inline]
fn handle_parse_err(error: ParseError){
    match error{
        ParseError::InvalidDate(date) => {
            println!("Invalid date: {}", date);
            start_menu();
        },
        ParseError::InvalidWeatherCode(code) => {
            println!("Invalid weather code: {}", code);
            start_menu();
        },
        ParseError::InvalidTemperature(temp) => {
            println!("Invalid temperature: {}", temp);
            start_menu();
        },
        ParseError::InvalidWind(wind) => {
            println!("Invalid wind: {}", wind);
            start_menu();
        },
        ParseError::InvalidPrecipitationProbability(prob) => {
            println!("Invalid precipitation probability: {}", prob);
            start_menu();
        },
        ParseError::InvalidPrecipitation(sum) => {
            println!("Invalid precipitation: {}", sum);
            start_menu();
        },
        ParseError::InvalidLine(line) => {
            println!("Invalid line: {}", line);
            start_menu();
        },
        ParseError::TooManyValues => {
            println!("Inconsistent number of values!");
            start_menu();
        },
        ParseError::DuplicateDate(date) => {
            println!("Duplicate date: {}", date.to_string());
            start_menu();
        }

    }
}


fn main() {
    start_menu();
}
