mod lib;

use lib::{run};

fn main() {
    let complete_path = std::env::current_dir().unwrap();
    let file_path = std::env::args().nth(1).unwrap();
    let input = std::fs::read_to_string(complete_path.join(file_path)).unwrap();
    let sanitized_input = input.split("\n").into_iter().map(|line| line.trim()).collect::<Vec<&str>>().join("");
    run(&sanitized_input);
}
