use std::fs::File;
use std::io::Read;

use parser::*;

mod parser;

fn main() {
    let path = "data/weather.csv";
    let mut data = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut data);
    
}
