use chrono::{self, Datelike, Timelike, Weekday};
use chrono::{DateTime, Local};
use std::fs;
use std::io;

const JOURNALS_ROOT_DIR: &str = "./Journals";

fn now() -> DateTime<Local> {
    Local::now()
}

fn datestamp(time: &DateTime<Local>) -> String {
    format!("{}/{}/{}", time.year(), time.month(), time.day())
}

fn timestamp(time: &DateTime<Local>) -> String {
    let (pm, hour) = time.hour12();
    let am_pm = if pm { "pm" } else { "am" };

    format!("{}:{} {}", hour, time.minute(), am_pm)
}

fn read_file(path: &str) -> io::Result<String> {
    return fs::read_to_string(path);
}

fn write_file(path_str: &str, contents: &str) {
    let path = std::path::Path::new(path_str);
    if let Some(prefix) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(prefix) {
            println!("Could not parent directories for {}: {}", path_str, e);

            return;
        }
    }

    if let Err(e) = fs::write(path, contents) {
        println!("Could not write journal {}: {}", path_str, e);
    }
}

fn journal_dir(name: &str, date: &DateTime<Local>) -> String {
    format!(
        "{}/{}/{}/{}/{}.txt",
        JOURNALS_ROOT_DIR,
        name,
        date.year(),
        date.month(),
        date.day()
    )
}

fn new_journal_text(name: &str, date: &DateTime<Local>) -> String {
    let ds = datestamp(date);
    let weekday = match date.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    };

    format!("{} - {} {}\n", name, weekday, ds)
}

fn append_journal_line(
    journal_text: String,
    date: &DateTime<Local>,
    indent: usize,
    contents: &str,
) -> String {
    let tabs = "\t".repeat(indent);
    let line: String = format!("{}{} - {}", &tabs, &timestamp(date), contents);
    let new_journal_text = format!("{}\n{}", &journal_text, line);

    return new_journal_text;
}

fn load_journal(name: &str, date: &DateTime<Local>) -> String {
    let dir: String = journal_dir(&name, &date);
    let time = timestamp(date);

    match read_file(&dir) {
        Ok(str) => str,
        _ => {
            println!("Couldn't read the contents of {} for {}", name, &time);
            String::from("")
        }
    }
}

fn save_journal(name: &str, date: &DateTime<Local>, text: &str) {
    let dir: String = journal_dir(&name, &date);

    write_file(&dir, &text);
}

fn get_input() -> String {
    let stdin = io::stdin();

    let mut input = String::from("");
    if let Err(_) = stdin.read_line(&mut input) {
        return String::from("");
    }

    input = input.trim_end().to_string();
    return input;
}

fn display_journal(name: &str) {
    let date = now();
    let content = load_journal(&name, &date);
    println!("{}", &content);
}

fn main() {
    let name: String = String::from("Personal");

    loop {
        display_journal(&name);

        let input = get_input();
        let date = now();

        // process input
        if input.starts_with("exit") {
            break;
        }

        let mut content = load_journal(&name, &date);
        content = match input {
            _ => {
                if content == "" {
                    content = new_journal_text(&name, &date);
                }
                append_journal_line(content, &date, 0, &input)
            }
        };

        save_journal(&name, &date, &content);
    }
}
