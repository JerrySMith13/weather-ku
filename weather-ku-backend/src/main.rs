use std::process;
use inquire::{Select, Confirm, Editor};
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
    println!("Parsing data from manual input");
    let editor_data = Editor::new("Enter data here:").prompt();
    match editor_data {
        Ok(data) => {
            data_options(data.as_str());
        },
        Err(_) => start_menu(),
    }

}

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

fn data_options(data: &str){
    println!("Data options");
}

fn main() {
    println!("TEST");
    start_menu();
}
