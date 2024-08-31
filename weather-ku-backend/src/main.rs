use std::process;

use inquire::{Select, Confirm};
mod parser;

fn exit_dialog(menu: fn()) {
    let exit = Confirm::new("Are you sure you want to exit?").prompt();
    match exit {
        Ok(true) => process::exit(0),
        Ok(false) => menu(),
        Err(_) => process::exit(1),
    }
}

fn start_menu() {
    let menu_options = vec!["Parse data from file", "Enter data manually", "Exit"];
    let select_option = Select::new("Welcome! Select an option:", menu_options)
    .prompt();

    match select_option {
        Ok(option) => {
            match option {
                "Parse data from file" => println!("Parsing data from file..."),
                "Enter data manually" => println!("Entering data manually..."),
                "Exit" => exit_dialog(start_menu),
                _ => println!("Invalid option!"),
            }
        },
        Err(_) => start_menu(),
    }
        
}

fn main() {
    start_menu();
}
