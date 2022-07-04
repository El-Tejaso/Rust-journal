use console::Term;
use std::{
    env, fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

fn read_file(path: &PathBuf) -> io::Result<String> {
    return fs::read_to_string(path);
}

fn write_file(path: &PathBuf, contents: &str) {
    if let Some(prefix) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(prefix) {
            println!("Could not parent directories for {:#?}: {}", path, e);

            return;
        }
    }

    if let Err(e) = fs::write(path, contents) {
        println!("Could not write log at {:#?}: {}", path, e);
    }
}

fn clear_screen() {
    let term = Term::stdout();
    term.clear_screen().expect("failed clearing screen");
}

fn get_input_str() -> String {
    let stdin = io::stdin();

    let mut input = String::from("");
    if let Err(_) = stdin.read_line(&mut input) {
        return String::from("");
    }

    input = input.trim_end().to_string();

    if input.starts_with("/") {
        let command = &input[1..];
        if command == "exit" {
            clear_screen();
            std::process::exit(0);
        }
    }

    return input;
}

fn read_list(path: &PathBuf) -> Vec<String> {
    // read the file with the list
    let contents = match read_file(path) {
        Ok(contents) => contents,
        Err(e) => {
            panic!("{}", &format!("Error: {:?}", e));
        }
    };

    //
    let list = contents
        .split("\n")
        .map(|x| String::from(x))
        .filter(|x| x.len() > 0)
        .collect::<Vec<String>>();

    return list;
}

fn write_list(list: &Vec<String>, path: &PathBuf) {
    let contents = list.join("\n");
    write_file(path, &contents);
}

fn print_help() {
    println!("
Syntax:
a (or append or add) [text]                Append to the end of the list (the default action)
e (or enqueue) [text]                      Enqueue at the start of the list
i (or insert) [number] [text]              Insert at a position in the list
x (or remove or rm) [number]               Remove from list at a position
m (or move) [number src] [number dst]      Move item in list to a position
r (or rename) [number] [text]              Rename item in list at a position
t (or tag) [number] [text]                 Add or change the tag of a task at a position
(negative numbers wrap back around)
"
    );
}

enum Command {
    Enqueue,
    Insert,
    Append,
    Remove,
    Move,
    Rename,
    Tag
}

fn main() {
    // Get the file we want to open
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: list [file]");
    }
    let mut path = PathBuf::from(args[1].clone());
    if path.extension() == None {
        path.set_extension("txt");
    }

    // initialize the file if it doesn't already exist
    match read_file(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                let name = path.file_name().expect("file must have a name");
                let content = format!("list - [{}]\n", name.to_string_lossy());
                write_file(&path, &content);
            } else {
                panic!("Couldn't open file");
            }
        }
        _ => {}
    }

    fn parse_and_wrap(input: &str, len: usize) -> Option<usize> {
        if let Ok(mut num) = input.parse::<i32>() {
            if num == 0 {
                println!(" Only enter positive or negative numbers as indices\n");
                return None;
            }

            if num > 0 {
                num -= 1;
            }

            if num < 0 {
                num = (len as i32) + num;
            }

            if num < 0 {
                return None;
            }

            if num >= (len as i32) {
                return None;
            }

            return Some(num as usize);
        }


        return None;
    }

    let mut failed = false;

    // REPL
    loop {
        // print
        let mut list = read_list(&path);
        print_list(&path.file_name().unwrap().to_string_lossy(), &list);

        // also print error from last run. We do it here so it appears after the list
        if failed {
            print_help();
        }

        failed = false;

        // read
        let original_input = get_input_str();

        if original_input == "help" {
            failed = true;
            continue;
        }

        // evaluate
        clear_screen();
        match original_input.split_once(' ') {
            Some((command_str, input)) => {
                let mut input_raw = input;
                let mut input = String::from(input);

                let command = match command_str {
                    "e" | "enqueue" => Command::Enqueue,
                    "a" | "add" | "append" => Command::Append,
                    "x" | "rm" | "remove" => Command::Remove,
                    "m" | "move" => Command::Move,
                    "i" | "insert" => Command::Insert,
                    "t" | "tag" => Command::Tag,
                    "r" | "rename" => Command::Rename,
                    _ => {
                        input = original_input.clone();
                        input_raw = &input[..];

                        Command::Append
                    }
                };

                match command {
                    Command::Enqueue => {
                        list.insert(0, input);
                        failed = false;
                    }
                    Command::Append => {
                        list.push(input);
                        failed = false;
                    }
                    Command::Remove => {
                        if let Some(index) = parse_and_wrap(input_raw, list.len()) {
                            if index < list.len() {
                                list.remove(index);
                                failed = false;
                            }
                        }
                    }
                    Command::Move => {
                        if let Some((src, dst)) = input.split_once(' ') {
                            if let Some(src_idx) = parse_and_wrap(src, list.len()) {
                                if let Some(dst_idx) = parse_and_wrap(dst, list.len()) {
                                    if src_idx < list.len() && dst_idx < list.len() {
                                        let item = list.remove(src_idx);
                                        list.insert(dst_idx, item);
                                        failed = false;
                                    }
                                }
                            }
                        }
                    }
                    Command::Insert => {
                        if let Some((dst, item)) = input.split_once(' ') {
                            if let Some(dst_idx) = parse_and_wrap(dst, list.len()) {
                                if dst_idx < list.len() {
                                    list.insert(dst_idx, String::from(item));
                                    failed = false;
                                }
                            }
                        }
                    }
                    Command::Rename => {
                        if let Some((dst, item)) = input.split_once(' ') {
                            if let Some(dst_idx) = parse_and_wrap(dst, list.len()) {
                                if dst_idx < list.len() {
                                    list[dst_idx] = String::from(item);
                                    failed = false;
                                }
                            }
                        }
                    }
                    Command::Tag => {
                        if let Some((dst, item)) = input.split_once(' ') {
                            if let Some(dst_idx) = parse_and_wrap(dst, list.len()) {
                                if dst_idx < list.len() {
                                    let mut current_item = &list[dst_idx][..];
                                    if current_item.contains('|') {
                                        current_item = &current_item[0..current_item.find('|').unwrap()];
                                    }

                                    if item != "x" {
                                        list[dst_idx] = format!("{} | {}", current_item.trim(), item);
                                    } else {
                                        list[dst_idx] = format!("{}", current_item.trim())
                                    }
                                        
                                    failed = false;
                                }
                            }
                        }
                    }
                }
            }
            None => {}
        }

        if failed {
            continue;
        }

        write_list(&list, &path);
    }
}

fn print_list(filename: &str, list: &Vec<String>) {
    let header = format!(
        "\n________________________________ {} ________________________________",
        filename
    );
    println!("{}", header);
    println!("{} |", " ".repeat(pad(0, list.len()).len()));
    
    // this is where we will indent the tags to 
    let width = header.len() - 10;

    for (i, line) in list.iter().enumerate() {
        print!("{} | ", &pad(i + 1, list.len()));

        if line.contains('|') {
            let (text, tag) = line.split_once('|').unwrap();

            if text.len() < width {
                println!("{} <{} {}", text, "-".repeat(width - text.len()) ,tag);
            } else {
                println!("{} <{} {}", text, "--" ,tag);
            }
        } else {
            println!("{}", line);
        }

    }

    println!("\n<end of list>\n")
}

// hard coded. I can't Rust
fn pad(index: usize, max_index: usize) -> String {
    if max_index < 10 {
        return format!("{}", index);
    }

    if max_index < 100 {
        if index < 10 {
            return format!("{} ", index);
        }

        return format!("{}", index);
    }

    if index < 10 {
        return format!("{}  ", index);
    }

    if index < 100 {
        return format!("{} ", index);
    }

    return format!("{}", index);
}
