use indexmap::IndexMap;
use inquire::{Confirm, Editor, InquireError, Select};
use parser::{Date, ParseError, WeatherData};
use std::process;

mod pathfinder;

#[derive(Eq, PartialEq, Clone, Copy)]
enum DataPoint {
    WeatherCode,
    HighTemperature,
    LowTemperature,
    TotalPrecipitation,
    HighestPrecipitationChance,
    MaximumWindSpeed,
}
impl DataPoint{
    fn to_string(&self) -> String{
        match self{
            DataPoint::WeatherCode => "Weather Code".to_string(),
            DataPoint::HighTemperature => "High Temperature".to_string(),
            DataPoint::LowTemperature => "Low Temperature".to_string(),
            DataPoint::TotalPrecipitation => "Total Precipitation".to_string(),
            DataPoint::HighestPrecipitationChance => "Highest Precipitation Chance".to_string(),
            DataPoint::MaximumWindSpeed => "Maximum Wind Speed".to_string(),
        }
    }
}
// OPERATIONS \\
#[inline]
fn avg(set: Vec<f32>) -> f32 {
    let mut sum = 0.0;
    let mut len = 0;
    for val in set {
        sum += val;
        len += 1;
    }
    sum / len as f32
}

#[inline]
fn min(set: Vec<f32>) -> f32 {
    let mut min = set[0];
    for val in set {
        if val < min {
            min = val;
        }
    }
    min
}

#[inline]
fn max(set: Vec<f32>) -> f32 {
    let mut max = set[0];
    for val in set {
        if val > max {
            max = val;
        }
    }
    max
}

// END OPERATIONS \\
fn exit_dialog(menu: fn()) {
    let exit = Confirm::new("Are you sure you want to exit?").prompt();
    match exit {
        Ok(true) => process::exit(0),
        Ok(false) => menu(),
        Err(_) => process::exit(1),
    }
}
// INPUT METHODS \\
fn data_from_file(path: String) {
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            println!("Error: {}", e);
            start_menu();
            return;
        }
    };

    let weather_data = match WeatherData::from_data(data) {
        Ok(data) => data,
        Err(e) => {
            handle_parse_err(e);
            return;
        }
    };
    get_options(weather_data);
    
}

//FIXME
fn data_from_manual() {
    let editor_data = Editor::new("Enter data here:")
        .with_help_message("Save and close the file to submit!")
        .prompt();
    match editor_data {
        Ok(data) => {
            (data.as_str());
        }
        Err(_) => start_menu(),
    }
}
//END INPUT METHODS\\

fn start_menu() {
    let menu_options = vec!["Parse data from file", "Enter data manually", "Exit"];
    let select_option = Select::new("Welcome! Select an option:", menu_options).prompt();

    match select_option {
        Ok(option) => match option {
            "Parse data from file" => data_from_file(pathfinder::file_dialog(".")),
            "Enter data manually" => data_from_manual(),
            "Exit" => exit_dialog(start_menu),
            _ => println!("Invalid option!"),
        },
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
        }
        Err(_) => start_menu(),
    }
}

fn data_ops(data: IndexMap<Date, WeatherData>, point: DataPoint) {
    let range = date_range(&data);
    let mut set = Vec::with_capacity(range.len());
    for data in range.values() {
        match point {
            DataPoint::WeatherCode => {set.push(data.weather_code as f32);}
            DataPoint::HighTemperature => set.push(data.temp_max),
            DataPoint::LowTemperature => set.push(data.temp_min),
            DataPoint::TotalPrecipitation => set.push(data.precip_sum),
            DataPoint::HighestPrecipitationChance => set.push(data.precip_prob_max),
            DataPoint::MaximumWindSpeed => set.push(data.max_wind),
        }
    }
    let options: Vec<&str>;
    if range.len() == 1 || point == DataPoint::WeatherCode {
        options = vec!["Single Point"];
    } else {
        options = vec!["Single Point", "Average", "Minimum", "Maximum"];
    }
    let op = match Select::new("Select an operation to perform: ", options).prompt() {
        Ok(op) => op,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
            return;
        }
        Err(_) => {
            println!("Error occured, please try again.");
            start_menu();
            return;
        }
    };
    let message = match op {
        "Single Point" => {single_point_select(range, point)}
        "Average" => {
            let avg = avg(set);
            format!("Average {} from {} to {}: {}", point.to_string(), range.first().unwrap().0.to_string(), range.last().unwrap().0.to_string(), avg)
        }
        "Minimum" => {
            let min: f32 = min(set);
            format!("Average {} from {} to {}: {}", point.to_string(), range.first().unwrap().0.to_string(), range.last().unwrap().0.to_string(), min)
        }
        "Maximum" => {
            let max = max(set);
            format!("Average {} from {} to {}: {}", point.to_string(), range.first().unwrap().0.to_string(), range.last().unwrap().0.to_string(), max)
        }
        _ => {
            println!("Invalid option! Please try again");
            data_ops(data, point);
            return;
        }
    };
    let final_menu = Select::new(message.as_str(), vec!["Try Different Operation", "Select New Datapoint", "Return To Main Menu", "Exit"]).prompt();
    match final_menu {
        Ok(option) => match option {
            "Try Different Operation" => data_ops(data, point),
            "Select New Datapoint" => get_options(data),
            "Return To Main Menu" => start_menu(),
            "Exit" => exit_dialog(start_menu),
            _ => {
                println!("Invalid option! Please try again");
                data_ops(data, point);
            }
        },
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
        }
        Err(_) => start_menu(),
    }
}

fn single_point_select(range: IndexMap<Date, &WeatherData>, point: DataPoint) -> String {
    let point_str = point.to_string();
    let options = range.keys().map(|date| date.to_string()).collect();
    let date = match Select::new("Select a date to sample: ", options).prompt() {
        Ok(date) => date,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
            return "".to_string();
        }
        Err(_) => {
            println!("Error occured, please try again.");
            start_menu();
            return "".to_string();
        }
    };
    let data = range.get(&Date::from_string(&date).unwrap()).unwrap();
    let data = match point {
        DataPoint::WeatherCode => data.weather_code.to_string(),
        DataPoint::HighTemperature => data.temp_max.to_string(),
        DataPoint::LowTemperature => data.temp_min.to_string(),
        DataPoint::TotalPrecipitation => data.precip_sum.to_string(),
        DataPoint::HighestPrecipitationChance => data.precip_prob_max.to_string(),
        DataPoint::MaximumWindSpeed => data.max_wind.to_string(),
    };
    return format!("{} for {}: {}", point_str, date, data);
}

fn get_options(data: IndexMap<Date, WeatherData>) {
    
    let options = vec![
        "Weather Code",
        "High Temperature",
        "Low Temperature",
        "Total Precipitation",
        "Highest Precipitation Chance",
        "Maximum Wind Speed",
    ];
    let select = match Select::new("Select a data point to sample:", options).prompt() {
        Ok(option) => option,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
            return;
        }
        Err(_) => {
            println!("Error occured, please try again.");
            start_menu();
            return;
        }
    };
    match select {
        "Weather Code" => {data_ops(data, DataPoint::WeatherCode);}
        "High Temperature" => {data_ops(data, DataPoint::HighTemperature);}
        "Low Temperature" => {data_ops(data, DataPoint::LowTemperature);}
        "Total Precipitation" => {data_ops(data, DataPoint::TotalPrecipitation);}
        "Highest Precipitation Chance" => {data_ops(data, DataPoint::HighestPrecipitationChance);}
        "Maximum Wind Speed" => {data_ops(data, DataPoint::MaximumWindSpeed);}
        _ => {
            println!("Invalid option! Please try again");
            get_options(data);
        }
    }
}

fn date_range(data: &IndexMap<Date, WeatherData>) -> IndexMap<Date, &WeatherData> {
    let mut dates_to_display: Vec<String> = Vec::with_capacity(data.len());
    for node in data.values() {
        dates_to_display.push(node.date.to_string());
    }
    let begin_date = match Select::new("Begin date: ", dates_to_display.clone()).prompt() {
        Ok(date) => date,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
            return IndexMap::new();
        }
        Err(_) => {
            println!("Error occured, please try again.");
            start_menu();
            return IndexMap::new();
        }
    };

    // Ensures that only selectable dates are displayed
    for i in 0..dates_to_display.len() {
        if dates_to_display[i] == begin_date {
            dates_to_display.drain(0..i);
            break;
        }
    }

    let end_date = match Select::new(
        format!(
            "Begin date: {} | End date: ",
            begin_date.clone().to_string()
        )
        .as_str(),
        dates_to_display,
    )
    .prompt()
    {
        Ok(date) => date,
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            exit_dialog(start_menu);
            return IndexMap::new();
        }
        Err(_) => {
            println!("Error occured, please try again.");
            start_menu();
            return IndexMap::new();
        }
    };

    let start_point = data
        .get_index_of(&Date::from_string(&begin_date).unwrap())
        .unwrap();
    let end_point = data
        .get_index_of(&Date::from_string(&end_date).unwrap())
        .unwrap();
    let range = data.get_range(start_point..end_point + 1).unwrap();
    let range: IndexMap<Date, &WeatherData> = range
        .into_iter()
        .map(|(date, data)| (date.clone(), data))
        .collect();
    range
}

#[inline]
fn handle_parse_err(error: ParseError) {
    print!("Error! ");
    match error {
        ParseError::InvalidDate(date) => {
            println!("Invalid date: {}", date);
            start_menu();
        }
        ParseError::InvalidWeatherCode(code) => {
            println!("Invalid weather code: {}", code);
            start_menu();
        }
        ParseError::InvalidTemperature(temp) => {
            println!("Invalid temperature: {}", temp);
            start_menu();
        }
        ParseError::InvalidWind(wind) => {
            println!("Invalid wind: {}", wind);
            start_menu();
        }
        ParseError::InvalidPrecipitationProbability(prob) => {
            println!("Invalid precipitation probability: {}", prob);
            start_menu();
        }
        ParseError::InvalidPrecipitation(sum) => {
            println!("Invalid precipitation: {}", sum);
            start_menu();
        }
        ParseError::InvalidLine(line) => {
            println!("Invalid line: {}", line);
            start_menu();
        }
        ParseError::TooManyValues => {
            println!("Inconsistent number of values!");
            start_menu();
        }
        ParseError::DuplicateDate(date) => {
            println!("Duplicate date: {}", date.to_string());
            start_menu();
        }
    }
}

fn main() {
    start_menu();
}
