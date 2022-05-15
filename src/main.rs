use chrono::{self, Datelike, Duration, Timelike, Weekday};
use chrono::{DateTime, Local};
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::str::FromStr;

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

fn journal_root_dir(name: &OsStr) -> PathBuf {
    let mut path = PathBuf::new();

    path.push(JOURNALS_ROOT_DIR);
    path.push(name);

    return path;
}

fn journal_dir(name: &OsStr, date: &DateTime<Local>) -> PathBuf {
    let mut path = journal_root_dir(name);

    path.push(format!("{}", date.year()));
    path.push(format!("{}", &two_dig_number(date.month())));
    path.push(format!("{}.txt", &two_dig_number(date.day())));

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

fn load_journal_err(name: &OsStr, date: &DateTime<Local>) -> Result<String, std::io::Error> {
    let dir: PathBuf = journal_dir(&name, &date);

    return read_file(&dir);
}

// This will initialize a journal if not present.
fn load_journal(name: &OsStr, date: &DateTime<Local>) -> String {
    let datestamp = datestamp(&date);
    return match load_journal_err(name, date) {
        Ok(str) => str,
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                let text = new_journal_text(name, date);
                save_journal(name, date, &text);
                return text;
            }

            panic!(
                "Couldn't read the contents journal {} for {}: {}",
                name.to_string_lossy(),
                &datestamp,
                e
            );
        }
    };
}

fn save_journal(name: &OsStr, date: &DateTime<Local>, text: &str) {
    let dir: PathBuf = journal_dir(&name, &date);

    write_file(&dir, &text);
}

fn get_input_str() -> String {
    let stdin = io::stdin();

    let mut input = String::from("");
    if let Err(_) = stdin.read_line(&mut input) {
        return String::from("");
    }

    input = input.trim_end().to_string();

    if input == "/exit" {
        clear_screen();
        std::process::exit(0);
    }

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
        Ok(journals) => pick_journal_from_existing(&journals),
        Err(_) => pick_new_journal_name(),
    };

    return name;
}

fn pick_new_journal_name() -> OsString {
    loop {
        clear_screen();
        println!("Enter the name of your new journal:");
        let name = get_input_str();

        if let Ok(available_journals) = get_journals() {
            if let Some(existing_name) = find_journal(&name, &available_journals) {
                println!(
                    "That name already refers to the journal '{}', please pick another one.",
                    existing_name.to_string_lossy()
                );
            }
        }

        let name = OsString::from(name);
        let date = now();

        if let Err(_) = load_journal_err(&name, &date) {
            save_journal(&name, &date, &new_journal_text(&name, &date));
        }

        return name;
    }
}

fn pick_journal_from_existing(journals: &Vec<OsString>) -> OsString {
    loop {
        clear_screen();
        print_available_journals(&journals);

        let input = get_input_str();
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

fn clear_screen() {
    print!("{}c", 27 as char);
}

fn main() {
    let mut name = pick_journal();
    let mut err = String::from("");

    loop {
        clear_screen();
        display_journal(&name);

        if err.len() > 0 {
            println!("{}", &err);
            err = String::from("");
        }

        let input = get_input_str();
        let date = now();

        // process input
        if input.trim() == "" {
            continue;
        } else if input.trim() == "-" {
            continue;
        } else if input.starts_with("/set") {
            if let Ok(available_journals) = get_journals() {
                if input.len() == "/set".len() {
                    name = pick_journal_from_existing(&available_journals);
                }
            } else {
                err = String::from("No journals available");
            }
            continue;
        } else if input == "/new" {
            name = pick_new_journal_name();
            continue;
        } else if input.starts_with("/last") || input.starts_with("/prev") {
            display_prev_journals_input_loop(&name, &date, 20);
            continue;
        } else if input.starts_with("/time") {
            todo!();
        } else if input.starts_with("/") {
            continue;
        }

        let new_content = append_to_journal(&name, date, input);

        save_journal(&name, &date, &new_content);
    }
}

fn display_prev_journals(name: &OsStr, date: &DateTime<Local>, page_size: u32, mut page_num: u32) {
    clear_screen();
    if page_num > 0 {
        page_num -= 1;
    }

    let start = page_size * page_num;
    let end = start + page_size;

    let mut count = 0;
    let mut current_date = date.clone();

    let total_num_years: i64;
    match get_journal_num_years(name) {
        Ok(num) => {
            if num == 0 {
                println!(
                    "You don't have any entries in the journal {}",
                    name.to_string_lossy()
                );
                return;
            }

            total_num_years = num as i64;
        }
        Err(e) => {
            println!("Couldn't find previous journals: {}", e);
            return;
        }
    }

    let mut journals: Vec<String> = Vec::new();
    // not particularly efficient at all, but a lot simpler to code, and viable because
    // I don't think anyone will ever write enough journals in their lifetimes to make this algorithm
    // noticeably slow
    loop {
        if let Ok(journal) = load_journal_err(name, &current_date) {
            if count >= start {
                journals.push(journal);
            }

            count += 1;
        }

        if count >= end {
            break;
        }

        current_date = current_date - Duration::days(1);
        if (date.signed_duration_since(current_date).num_days() / 365) > total_num_years {
            break;
        }
    }

    journals.reverse();
    for (i, journal_text) in journals.iter().enumerate() {
        if page_num == 0 && i == (journals.len() - 1) {
            println!("\n\n\n---------------- <latest entry> ----------------\n\n");
        } else {
            println!(
                "\n\n\n---------------- <latest - {}> ----------------\n\n",
                journals.len() - 1 - i 
            );
        }

        println!("{}", &journal_text);
    }

    println!("\n\n");
    if journals.len() == 0 {
        println!(
            "No entries were found for page {} with a page size of {}.",
            page_num, page_size
        );
    } else {
        println!(
            "Viewing the last {} to {} entries",
            start + 1,
            start + 1 + (journals.len() as u32)
        );
        if journals.len() != page_size as usize {
            println!("(Only {}/{} entries were found)", journals.len(), page_size);
        }
    }
}

fn get_journal_num_years(name: &OsStr) -> Result<usize, io::Error> {
    let root_dir = journal_root_dir(name);
    let dirs = std::fs::read_dir(root_dir)?;

    Ok(dirs.collect::<Vec<io::Result<std::fs::DirEntry>>>().len())
}

fn get_input<T: std::str::FromStr>(message: &str) -> Result<T, <T as FromStr>::Err> {
    println!("{}", message);
    let input = get_input_str();

    return input.parse::<T>();
}

fn display_prev_journals_input_loop(name: &OsStr, date: &DateTime<Local>, page_size: u32) {
    let mut page = 0;
    loop {
        display_prev_journals(name, date, page_size, page);
        if let Ok(page_num) =
            get_input::<u32>("input a page number (1 or more), or anything else to go back")
        {
            page = page_num;
        } else {
            break;
        }
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
