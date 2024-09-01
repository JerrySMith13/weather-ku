use inquire::Select;
use std::fs::{read_dir, DirEntry};
use std::collections::HashMap;

enum FileOption{
    Going(String),
    Done(String),
}

fn file_path_add(path_to_add: &str) -> FileOption{
    let files = match read_dir(path_to_add){
        Ok(files) => files,
        Err(e) => {
            panic!("Error: {}", e);
        }
    };

    let file_list: Vec<_> = files.map(|f| f.unwrap()).collect();  

    let mut file_map: HashMap<String, DirEntry> = HashMap::new();
    for file in file_list {
        let file_name = file.file_name().into_string().unwrap();
        file_map.insert(file_name, file);
    }

    let file_names: Vec<String> = file_map.keys().map(|k| k.clone()).collect();

    let msg = format!("Select a file from {}", path_to_add);
    let msg = msg.as_str();

    let select_menu = Select::new(msg, file_names);
    let selected_file = file_map.get(&select_menu.prompt().unwrap());

    match selected_file.unwrap().file_type().unwrap().is_dir(){
        true => FileOption::Going(selected_file.unwrap().file_name().to_str().unwrap().to_string()),
        false => FileOption::Done(selected_file.unwrap().file_name().to_str().unwrap().to_string()),
    
    }

}



pub fn file_dialog(path: &str) -> String{
    let mut current_path = path.to_string();
    loop{
        match file_path_add(&current_path){
            FileOption::Going(path) => {
                current_path.push_str(format!("/{}", path).as_str());
            },
            FileOption::Done(path) => {
                current_path.push_str(format!("/{}", path).as_str());
                return current_path;
            },
        }
    }   
    


}