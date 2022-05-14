use chrono::{self, Datelike, Timelike, Weekday};
use chrono::{DateTime, Local};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self};
use std::path::PathBuf;

const JOURNALS_ROOT_DIR: &str = "./Journals";

fn now() -> DateTime<Local> {
    Local::now()
}

fn datestamp(time: &DateTime<Local>) -> String {
    format!("{}/{}/{}", time.year(), time.month(), time.day())
}

fn two_dig_number(num: u32) -> String {
    if num < 10 {
        return format!("0{}", num);
    }

    return format!("{}", num);
}

fn timestamp(time: &DateTime<Local>) -> String {
    let (pm, hour) = time.hour12();
    let am_pm = if pm { "pm" } else { "am" };

    format!(
        "{}:{} {}",
        &two_dig_number(hour),
        &two_dig_number(time.minute()),
        am_pm
    )
}

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
        println!("Could not write journal {:#?}: {}", path, e);
    }
}

fn journal_dir(name: &OsStr, date: &DateTime<Local>) -> PathBuf {
    let mut path = PathBuf::new();

    path.push(JOURNALS_ROOT_DIR);
    path.push(name);
    path.push(format!("{}", date.year()));
    path.push(format!("{}", date.month()));
    path.push(format!("{}.txt", date.day()));

    return path;
}

fn new_journal_text(name: &OsStr, date: &DateTime<Local>) -> String {
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

    format!("{} - {} {}\n", name.to_string_lossy(), weekday, ds)
}

fn journal_line(date: &DateTime<Local>, indent: usize, contents: &str) -> String {
    let mut line = String::from("\n");

    for _i in 0..indent {
        line.push_str("\t");
    }

    line.push_str(&timestamp(&date));
    line.push_str(" - ");
    line.push_str(&contents);

    return line;
}

fn load_journal(name: &OsStr, date: &DateTime<Local>) -> String {
    let dir: PathBuf = journal_dir(&name, &date);
    let date = datestamp(&date);

    if !dir.exists() {
        return String::from("");
    }

    match read_file(&dir) {
        Ok(str) => str,
        Err(e) => {
            panic!(
                "Couldn't read the contents of {:#?} for {}: {}",
                name, &date, e
            );
        }
    }
}

fn save_journal(name: &OsStr, date: &DateTime<Local>, text: &str) {
    let dir: PathBuf = journal_dir(&name, &date);

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

fn display_journal(name: &OsStr) {
    let date = now();
    let content = load_journal(&name, &date);
    println!("{}", &content);
}

fn get_journals() -> Result<Vec<OsString>, io::Error> {
    let mut dirs: Vec<OsString>;

    match std::path::Path::new(JOURNALS_ROOT_DIR).read_dir() {
        Err(e) => {
            return Err(e);
        }
        Ok(dir_entries) => {
            dirs = Vec::new();
            for dir_entry in dir_entries {
                match dir_entry {
                    Ok(dir) => {
                        let path = dir.path();
                        if !path.is_file() {
                            if let Some(filename) = path.file_name() {
                                dirs.push(OsString::from(filename));
                            }
                        }
                    }
                    _ => {}
                }
            }

            if dirs.len() == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "There are no journals.",
                ));
            }

            return Ok(dirs);
        }
    }
}

fn pick_journal() -> OsString {
    let available_journals = get_journals();
    let name = match available_journals {
        Ok(journals) => pick_journal_from_existing(journals),
        Err(_) => pick_new_journal_name(),
    };

    let date = now();
    let content = load_journal(&name, &date);
    if content == "" {
        save_journal(&name, &date, &new_journal_text(&name, &date));
    }

    return name;
}

fn pick_new_journal_name() -> OsString {
    loop {
        println!("Enter the name of your new journal:");
        let name = get_input();

        if let Ok(available_journals) = get_journals() {
            if let Some(existing_name) = find_journal(&name, &available_journals) {
                println!(
                    "That name already refers to the journal {}, please pick another one.",
                    existing_name.to_string_lossy()
                );
            }
        }

        return OsString::from(name);
    }
}

fn pick_journal_from_existing(journals: Vec<OsString>) -> OsString {
    loop {
        print_available_journals(&journals);

        let input = get_input();
        if let Some(value) = find_journal(&input, &journals) {
            return value;
        }

        println!("Input was invalid, try again");
    }
}

fn print_available_journals(journals: &Vec<OsString>) {
    println!("Select a journal:");
    for (i, journal_name) in journals.iter().enumerate() {
        println!("[{}] - {}", i, journal_name.to_string_lossy());
    }
}

fn find_journal(input: &str, journals: &Vec<OsString>) -> Option<OsString> {
    match usize::from_str_radix(&input, 10) {
        Ok(index) => {
            if index < journals.len() {
                return Some(OsString::from(&journals[index]));
            }
        }
        Err(_) => {
            let input_lower = input.to_ascii_lowercase();
            for name in journals {
                let name_lower = String::from(name.to_string_lossy()).to_ascii_lowercase();
                println!("name_lower {}", name_lower);
                if name_lower.starts_with(&input_lower) {
                    return Some(OsString::from(name));
                }
            }
        }
    }

    return None;
}

fn main() {
    let mut name = pick_journal();
    let mut err = String::from("");

    loop {
        display_journal(&name);

        if err.len() > 0 {
            println!("{}", &err);
            err = String::from("");
        }

        let input = get_input();
        let date = now();

        // process input
        if input.trim() == "" {
            continue;
        } else if input.starts_with("-") && input.trim() == "-" {
            continue;
        } else if input.starts_with("exit") {
            break;
        } else if input.starts_with("set ") {
            match get_journals() {
                Ok(journals) => {
                    let rest = &input[4..];
                    if let Some(journal_name) = find_journal(rest, &journals) {
                        name = journal_name;
                    } else {
                        err = format!("{} not found.", rest);
                    }

                    continue;
                },
                _ => {

                }
            }
        }

        let new_content = append_to_journal(&name, date, input);

        save_journal(&name, &date, &new_content);
    }
}

fn append_to_journal(name: &OsStr, date: DateTime<Local>, input: String) -> String {
    let mut content = load_journal(name, &date);

    if input.starts_with("-") {
        push_block(date, &input, &mut content);
    } else if input == "~" {
        toggle_block(&mut content);
    } else {
        push_line(date, input, &mut content);
    }
    content
}

fn push_block(date: DateTime<Local>, input: &String, content: &mut String) {
    let new_line = journal_line(&date, 0, input[1..].trim());
    content.push_str("\n");
    content.push_str(&new_line);
}

fn push_line(date: DateTime<Local>, input: String, content: &mut String) {
    let new_line = journal_line(&date, 1, input.trim());
    content.push_str(&new_line);
}

fn toggle_block(content: &mut String) {
    let a = content.rfind("\n\n");
    let b = content.rfind("\n\t");
    if let Some(newlines) = a {
        if let Some(nl_tab) = b {
            if newlines < nl_tab {
                let mut new_content = String::from("");
                new_content.push_str(&content[0..nl_tab]);
                new_content.push_str("\n\n");
                new_content.push_str(&content[(nl_tab + 2)..]);
                content.clear();
                content.push_str(&new_content[..]);
            } else {
                let mut new_content = String::from("");
                new_content.push_str(&content[0..newlines]);
                new_content.push_str("\n\t");
                new_content.push_str(&content[(newlines + 2)..]);
                content.clear();
                content.push_str(&new_content[..]);
            }
        }
    }
}
