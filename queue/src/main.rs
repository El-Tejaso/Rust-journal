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
        .collect::<Vec<String>>();

    return list;
}

fn write_list(list: &Vec<String>, path: &PathBuf) {
    let contents = list.join("\n");
    write_file(path, &contents);
}

fn print_help() {
    println!(
        "Syntax:
e (or enqueue) [text]                      Enqueue at the start of the list (the default action)
i (or insert) [number] [text]              Insert at a position in the list
a (or append or add) [text]                Add to the end of the list
r (or remove) [number]                     Remove from list (negative numbers wrap back around)
m (or move) [number src] [number dst]      Move item in list
"
    );
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

    // REPL
    loop {
        // print
        let mut list = read_list(&path);
        print_list(&path.file_name().unwrap().to_string_lossy(), &list);

        // read
        let input = get_input_str();

        // evaluate
        clear_screen();
        let mut failed = true;
        match input.split_once(' ') {
            Some((command, input)) => {
                let input_raw = input;
                let input = String::from(input);

                match command {
                    "e" | "enqueue" => {
                        list.insert(0, input);
                        failed = false;
                    }
                    "a" | "add" | "append" => {
                        list.push(input);
                        failed = false;
                    }
                    "r" | "remove" => {
                        if let Some(index) = parse_and_wrap(input_raw, list.len()) {
                            if index < list.len() {
                                list.remove(index);
                                failed = false;
                            }
                        }
                    }
                    "m" | "move" => {
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
                    "i" | "insert" => {
                        if let Some((dst, item)) = input.split_once(' ') {
                            if let Some(dst_idx) = parse_and_wrap(dst, list.len()) {
                                if dst_idx < list.len() {
                                    list.insert(dst_idx, String::from(item));
                                    failed = false;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            None => {}
        }

        if failed {
            print_help();
            continue;
        }

        write_list(&list, &path);
    }
}

fn print_list(filename: &str, list: &Vec<String>) {
    println!(
        "\n________________________________ {} ________________________________",
        filename
    );
    println!("{} |", " ".repeat(pad(0, list.len()).len()));

    for (i, line) in list.iter().enumerate() {
        println!("{} | {}", &pad(i + 1, list.len()), line);
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
