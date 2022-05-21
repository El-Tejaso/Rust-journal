use chrono::{self, Datelike, Duration, Timelike, Weekday};
use chrono::{DateTime, Local};
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
use std::str::FromStr;
use console::Term;

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
    } else if input.starts_with("?")
        || input.starts_with("/?")
        || input.starts_with("?")
        || input.starts_with("help")
        || input.starts_with("/help")
    {
        print_help();
        return String::from("");
    }

    return input;
}

fn display_journal(name: &OsStr) {
    let date = now();
    let content = load_journal(&name, &date);

    let has_no_entries = content.matches("-").count() < 2;
    if has_no_entries {
        println!(
            "You haven't put any entries in [{}] yet.\nType '/help' at any time to find out how.\n\n",
            name.to_string_lossy()
        );
    }

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
        if journals.len() == 1 {
            return OsString::from(&journals[0]);
        }

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
    let term = Term::stdout();
    term.clear_screen().expect("failed clearing screen");
}

fn main() {
    let mut name = pick_journal();
    let mut message = String::from("");

    loop {
        clear_screen();
        display_journal(&name);

        if message.len() > 0 {
            println!("\n{}\n", &message);
            message = String::from("");
        }

        print!("\ncurrent->{}: ", &name.to_string_lossy());
        if let Err(_) = io::stdout().flush() {
            //guyse idk how to handle this one
        }

        let input = get_input_str();
        let date = now();

        // process input
        if input.trim() == "" || input.trim() == "-" {
            continue;
        } else if input.trim().starts_with("/") {
            if input.starts_with("/set") || input.starts_with("/switch") {
                if let Ok(available_journals) = get_journals() {
                    name = pick_journal_from_existing(&available_journals);
                } else {
                    // ideally, this line is never ever reached. I am not sure what the best flow here is
                    message = String::from("No journals available, use /new to make one.");
                }
            } else if input == "/new" {
                name = pick_new_journal_name();
            } else if input.starts_with("/last") || input.starts_with("/prev") {
                display_prev_journals_input_loop(&name, &date, 20);
            } else if input.starts_with("/time") {
                display_time_stats(&name, &date, false);
            } else if input.starts_with("/gtime") {
                display_time_stats(&name, &date, true);
            }

            continue;
        } else if input.starts_with("help") {
            continue;
        }

        match append_to_journal(&name, date, input) {
            Ok(new_content) => {
                save_journal(&name, &date, &new_content);
            }
            Err(e) => {
                message = e;
            }
        }
    }
}

fn print_help() {
    clear_screen();

    println!(
        "
        
Type /help from anywhere to access this text.

Type /exit from anywhere to exit this program.

        Journal Writing

Type any text to add an 'entry'. 
New entries will be added to the current 'block' of lines.

Type a dash (-) followed by an entry start a new block.

Type a tilde (~) on it's own to toggle the last line between an entry and block.
    (Useful for when you forget to add a dash (-) at the start)

        Journal Reading

Type /prev to view previous entries. 
Type /times to view a time breakdown of how much time elapsed between each block.
Type /gtime to show a more granular (but much harder to read) time breakdown between each entry.

        Journal Managing

You can have multiple journals.
Type /new to create a new journal. You will be asked to provide a name. 
Type /switch to switch to another journal. This will only work if you have more than one journal.
"
    );

    println!("Press enter to continue...");
    get_input_str();
}

fn display_time_stats(name: &OsStr, date: &DateTime<Local>, granular: bool) {
    clear_screen();
    fn parse_time(line: &str, date: &DateTime<Local>) -> Option<DateTime<Local>> {
        let colon_pos = line.find(":")?;

        let mut hour = line[colon_pos - 2..colon_pos].parse::<u32>().ok()?;
        let minute = line[colon_pos + 1..colon_pos + 3].parse::<u32>().ok()?;

        if hour != 12 && &line[colon_pos + 4..colon_pos + 6] == "pm" {
            hour += 12;
        }

        let time = date.clone().with_hour(hour)?.with_minute(minute)?;

        return Some(time);
    }

    let text = load_journal(name, date);
    let mut times: Vec<(DateTime<Local>, &str)> = Vec::new();
    let mut start = 0;

    while let Some(len) = text[start..].find("\n") {
        let end = start + len;
        let line = &text[start..end];
        if let Some(time) = parse_time(line, date) {
            times.push((time, line));
        }

        start = end + 1;
    }

    let line = &text[start..];
    if let Some(time) = parse_time(line, date) {
        times.push((time, line));
    }

    times.push((now(), "<now>"));

    println!(
        "Viewing time breakdown{}:\n\n",
        match granular {
            false => "",
            true => " (granular)",
        }
    );

    if times.len() > 0 {
        println!("{}", times[0].1);
        let mut block_time = times[0].0.clone();

        for i in 1..times.len() {
            let is_block = times[i].1.find("\t") == None;
            if is_block {
                println!("");
            }

            if is_block || granular {
                let dt = times[i].0.signed_duration_since(times[i - 1].0);
                let dt_from_start = times[i].0.signed_duration_since(times[0].0);
                let dt_from_block = times[i].0.signed_duration_since(block_time);
                println!(
                    "\nelapsed:\t\tsince start: {:.2}h      since block: {:.2}h      since last: {:.2}h\n",
                    (dt_from_start.num_minutes() as f64) / 60.0,
                    (dt_from_block.num_minutes() as f64) / 60.0,
                    (dt.num_minutes() as f64) / 60.0
                );
            }

            if is_block {
                block_time = times[i].0.clone();
                println!("");
            }

            println!("{}", times[i].1);
        }
    }

    println!("\n\npress enter to go back ...");
    get_input_str();
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
        let entry_count = if page_num == 0 && i == (journals.len() - 1) {
            String::from("")
        } else {
            format!(" - {}", journals.len() - 1 - i)
        };

        println!(
            "\n\n\n---------------- <latest entry{}> ----------------\n\n",
            &entry_count
        );

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
            "Viewing entries from latest-{} to latest-{}",
            start + (journals.len() as u32) - 1,
            start
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

fn append_to_journal(name: &OsStr, date: DateTime<Local>, input: String) -> Result<String, String> {
    let mut content = load_journal(name, &date);
    let has_no_entries = content.matches("-").count() < 2;
    if input.trim() == "~" {
        if !has_no_entries {
            toggle_block(&mut content);
        } else {
            return Err(String::from("Can't use '~' when there aren't any entries"));
        }
    } else if input.starts_with("-") || has_no_entries {
        let mut input = input.trim();
        if input.starts_with("-") {
            input = &input[1..];
        }
        
        push_block(date, &input, &mut content);
    } else {
        push_line(date, input, &mut content);
    }

    Ok(content)
}

fn push_block(date: DateTime<Local>, input: &str, content: &mut String) {
    let new_line = journal_line(&date, 0, input.trim());
    content.push_str("\n");
    content.push_str(&new_line);
}

fn push_line(date: DateTime<Local>, input: String, content: &mut String) {
    let new_line = journal_line(&date, 1, input.trim());
    content.push_str(&new_line);
}

fn toggle_block(content: &mut String) {
    fn newlines_to_nl_tab(content: &mut String, newlines:usize) {
        let mut new_content = String::from("");
        new_content.push_str(&content[0..newlines]);
        new_content.push_str("\n\t");
        new_content.push_str(&content[(newlines + 2)..]);
        content.clear();
        content.push_str(&new_content[..]);
    }

    fn nl_tab_to_newlines(content: &mut String, nl_tab:usize) {
        let mut new_content = String::from("");
        new_content.push_str(&content[0..nl_tab]);
        new_content.push_str("\n\n");
        new_content.push_str(&content[(nl_tab + 2)..]);
        content.clear();
        content.push_str(&new_content[..]);
    }

    let a = content.rfind("\n\n");
    let b = content.rfind("\n\t");

    if a == None && b == None {
        return;
    }

    if let Some(newlines) = a {
        if let Some(nl_tab) = b {
            if newlines < nl_tab {
                nl_tab_to_newlines(content, nl_tab);
            } else {
                newlines_to_nl_tab(content, newlines);
            }
        } else {
            newlines_to_nl_tab(content, newlines);
        }
    } else {
        if let Some(nl_tab) = b {
            nl_tab_to_newlines(content, nl_tab);
        }
    }
}
