use std::{option, process};
use inquire::{Confirm, Editor, MultiSelect, Select};
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

fn data_from_file(path: String){
    let data = match std::fs::read_to_string(path){
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            start_menu();
            return;
        }
    };

    data_options(data.as_str());
}

fn data_from_manual(){
    let editor_data = Editor::new("Enter data here:").with_help_message("Save and close the file to submit!").prompt();
    match editor_data {
        Ok(data) => {
            data_options(data.as_str());
        },
        Err(_) => start_menu(),
    }

}

fn start_menu() {
    let menu_options = vec!["Parse data from file", "Enter data manually", "Exit"];
    let select_option = Select::new("Welcome! Select an option:", menu_options).without_filtering()
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

#[inline]
fn parse_option(option: &str){
    match option{
        "Max temperature" => {
            println!("Max temperature");
        },
        "Min temperature" => {
            println!("Min temperature");
        },
        "Single point" => {
            println!("Single point");
        },
        "Average temperature" => {
            println!("Average temperature");
        },
        "Menu" => {
            start_menu();
        },
        "Exit" => {
            exit_dialog(start_menu);
        },
        _ => {
            parse_option(option);
        },
    }
}
fn data_options(data: &str){
    println!("{}", data); 
    // max, min, single point, average
    let menu_options = vec!["Max temperature", "Min temperature", "Single point", "Average temperature", "Menu", "Exit"];
    let select_option = Select::new("Select an option:", menu_options).prompt();
    match select_option{
        Ok(option) => parse_option(option),
        Err(_) => start_menu(),
    }

}

fn date_range(data: &str) -> Vec<parser::WeatherData>{
    let data = parser::WeatherData::from_data(data.to_string());
    match data{
        Ok(data) => {
            let mut dates_to_display: Vec<String>= Vec::with_capacity(data.len());
            for node in data.values(){
                dates_to_display.push(node.date.to_string());
            }
            let select_options = match MultiSelect::new("Select a date:", dates_to_display).prompt(){
                Ok(options) => options,
                Err(_) => {
                    println!("Sorry! Error Occured");
                    start_menu();
                    return Vec::new();
                },
            };
            let mut selected_dates: Vec<parser::WeatherData> = Vec::with_capacity(select_options.len());
            for option in select_options{
                selected_dates.push(data.get(&parser::Date::from_string(option.as_str()).unwrap()).unwrap().clone());
            }
            selected_dates
                    
        },
        Err(err) => {handle_parse_err(err); return Vec::new();},
    }

    
}

#[inline]
fn handle_parse_err(error: parser::ParseError){
    match error{
        parser::ParseError::InvalidDate(date) => {
            println!("Invalid date: {}", date);
            start_menu();
        },
        parser::ParseError::InvalidWeatherCode(code) => {
            println!("Invalid weather code: {}", code);
            start_menu();
        },
        parser::ParseError::InvalidTemperature(temp) => {
            println!("Invalid temperature: {}", temp);
            start_menu();
        },
        parser::ParseError::InvalidWind(wind) => {
            println!("Invalid wind: {}", wind);
            start_menu();
        },
        parser::ParseError::InvalidPrecipitationProbability(prob) => {
            println!("Invalid precipitation probability: {}", prob);
            start_menu();
        },
        parser::ParseError::InvalidPrecipitation(sum) => {
            println!("Invalid precipitation: {}", sum);
            start_menu();
        },
        parser::ParseError::InvalidLine(line) => {
            println!("Invalid line: {}", line);
            start_menu();
        },
        parser::ParseError::TooManyValues => {
            println!("Inconsistent number of values!");
            start_menu();
        },
        parser::ParseError::DuplicateDate(date) => {
            println!("Duplicate date: {}", date.to_string());
            start_menu();
        }

    }
}

fn main() {
    start_menu();
}
