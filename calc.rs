use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check if argument is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <number>", args[0]);
        return;
    }
    
    let num: i32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Please provide a valid integer");
            return;
        }
    };
    
    println!("{}", num * 2);
}