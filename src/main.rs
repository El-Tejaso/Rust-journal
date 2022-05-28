use chrono::{self, Datelike, Duration, TimeZone, Timelike, Weekday};
use chrono::{DateTime, Local};
use console::Term;
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, ErrorKind, Write};
use std::num::ParseIntError;
use std::path::{self, Path, PathBuf};
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
        Ok(str) => str.replace("\r", ""),
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

fn get_folders(path: &Path) -> Result<Vec<OsString>, io::Error> {
    let mut dirs: Vec<OsString>;

    let dir_entries = path.read_dir()?;

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

    return Ok(dirs);
}

fn get_journals() -> Result<Vec<OsString>, io::Error> {
    let path = path::Path::new(JOURNALS_ROOT_DIR);
    let res = get_folders(path);

    if let Ok(ref journals) = res {
        if journals.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "There are no journals.",
            ));
        }
    }

    return res;
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
            } else if input.starts_with("/find") {
                find_input_loop(&name, &date);
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
Help - Saturday 2022/5/21


09:50 am - Before we start:
    09:50 am - Type / help from anywhere to access this text
    09:50 am - Type /exit from anywhere to exit this program

09:51 am - The basics
    09:51 am - Type any text to add an 'entry'
    09:51 am - new entries will be added to the current 'block' of lines
    09:51 am - like
    09:51 am - this

09:52 am - Type dash (-) followed by an entry to start a new block
    09:52 am - Type a (~) on it's own to toggle the last line between being part of a block vs being the start of a new block

09:53 am - (Useful for when you forget a (-) on the line you just entered

09:53 am - Journal Reading
    09:54 am - Type /prev to view previous entries
    09:54 am - Type /times to view a time breakdown of how much time elapsed between each block.
    09:54 am - Type /gtime to show a more granular (but much harder to read) time breakdown between each entry.

09:54 am - Journal Managing
    09:54 am - You can have multiple journals.
    09:54 am - Type /new to create a new journal. You will be asked to provide a name.
    09:54 am - Type /switch to switch to another journal. This will only work if you have more than one journal.
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

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Forwards,
    Backwards,
}

fn iterate_journals_dir(
    name: &OsStr,
    date: &DateTime<Local>,
    dir: Direction,
    mut iter_fn: impl FnMut(&DateTime<Local>, String) -> bool,
) {
    let mut current_date = date.clone();

    fn get_year_index(
        current_date: &DateTime<Local>,
        years: &Vec<OsString>,
    ) -> Result<usize, ParseIntError> {
        let current_year = current_date.year();
        for index in 0..years.len() {
            let year = years[index].to_string_lossy().parse::<i32>()?;
            if year == current_year {
                return Ok(index);
            } else if year > current_year {
                return Ok(index - 1);
            }
        }

        return Ok(years.len());
    }

    let root_dir = journal_root_dir(name);
    let valid_years = match get_folders(&root_dir) {
        Ok(folders) => folders,
        Err(_) => {
            return;
        }
    };

    let current_year_os_str = OsString::from(format!("{}", current_date.year()));
    let mut current_year_index = get_year_index(&current_date, &valid_years)
        .expect("Folders in the journal root must all be 4 digit years");

    fn start_of_year(year: i32) -> DateTime<Local> {
        return chrono::Local.ymd(year, 1, 1).and_hms(0, 0, 0);
    }

    fn end_of_year(year: i32) -> DateTime<Local> {
        start_of_year(year + 1) - Duration::days(1)
    }

    fn parse_year(str: &OsStr) -> i32 {
        str.to_string_lossy()
            .parse::<i32>()
            .expect("Why isn't this a year. What happened??")
    }

    if &valid_years[current_year_index] != &current_year_os_str {
        let new_year = parse_year(&valid_years[current_year_index]);
        current_date = end_of_year(new_year);
    }

    loop {
        let this_year = current_date.year();

        while this_year == current_date.year() {
            if let Ok(journal_text) = load_journal_err(name, &current_date) {
                if !iter_fn(&current_date, journal_text.replace("\r", "")) {
                    return;
                }
            }

            current_date = match dir {
                Direction::Forwards => current_date + Duration::days(1),
                Direction::Backwards => current_date - Duration::days(1),
            };
        }

        if dir == Direction::Backwards && current_year_index == 0 {
            return;
        }

        current_year_index = match dir {
            Direction::Forwards => current_year_index + 1,
            Direction::Backwards => current_year_index - 1,
        };

        if current_year_index >= valid_years.len() {
            return;
        }

        let new_year = parse_year(&valid_years[current_year_index]);
        current_date = match dir {
            Direction::Forwards => start_of_year(new_year),
            Direction::Backwards => end_of_year(new_year),
        };
    }
}

fn find_input_loop(name: &OsStr, date: &DateTime<Local>) {
    fn iteration(
        _name: &OsStr,
        _date: &DateTime<Local>,
        find_str: &str,
        journal_text: &str,
    ) -> bool {
        if journal_text.to_lowercase().find(&find_str.to_lowercase()) == None {
            return true;
        }

        fn print_highlights(line: &str, find_str_index: usize, symbol: char, count: usize) {
            print!("    ");

            let mut current_index = 0 as usize;
            for c in line.chars() {
                if c == '\t' {
                    print!("\t");
                } else {
                    print!(" ");
                }

                current_index += 1;
                if current_index == find_str_index {
                    break;
                }
            }

            for _ in 0..count {
                print!("{}", symbol);
            }

            println!();
        }

        // find the block where the text is.
        for block in journal_text.split("\n\n") {
            if let Some(_block_index) = block
                .to_ascii_lowercase()
                .find(&find_str.to_ascii_lowercase())
            {
                // print each line, and highlight the one containing the result
                for line in block.split("\n") {
                    match line
                        .to_ascii_lowercase()
                        .find(&find_str.to_ascii_lowercase())
                    {
                        Some(index) => {
                            println!("");
                            print_highlights(line, index, 'v', find_str.len());
                            println!("--> {}     <--", line);
                            print_highlights(line, index, '^', find_str.len());
                            println!("");
                        }
                        None => {
                            println!("    {}", line);
                        }
                    }
                }
            }
        }

        //print journal heading
        match journal_text.find("\n\n") {
            None => return true,
            Some(index) => println!("\n\nFound results in {}:\n", &journal_text[0..index]),
        }

        return false;
    }

    let mut current_date = date.clone();
    let mut find_str = String::from("");

    clear_screen();
    loop {
        if &find_str != "" {
            println!("Searching for \"{}\"", &find_str);
        }

        if let Ok(find_str_input) = get_input::<String>(
            "Enter search text, or \">\" to go forwards or backwards, or \":quit\" to go back",
        ) {
            clear_screen();
            let new_date;

            if find_str_input.trim() == "<" || find_str_input.trim() == "" {
                new_date = current_date - Duration::days(1);
                println!("searching backwards from {} ...", new_date);
                iterate_journals_dir(
                    name,
                    &new_date,
                    Direction::Backwards,
                    |date, journal_text| {
                        current_date = date.clone();
                        return iteration(name, date, &find_str, &journal_text);
                    },
                );
            } else if find_str_input.trim() == ">" {
                new_date = current_date + Duration::days(1);
                println!("searching forwards from {} ...", new_date);
                iterate_journals_dir(
                    name,
                    &new_date,
                    Direction::Forwards,
                    |date, journal_text| {
                        current_date = date.clone();
                        return iteration(name, date, &find_str, &journal_text);
                    },
                );
            } else if find_str_input.trim() == ":quit" {
                break;
            } else {
                find_str = find_str_input;
                current_date = date.clone();
            }
        }
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
    let mut journals: Vec<String> = Vec::new();

    iterate_journals_dir(
        name,
        date,
        Direction::Backwards,
        |_current_date, journal_text| {
            if count >= start {
                journals.push(journal_text);
            }

            count += 1;

            if count >= end {
                return false;
            }

            return true;
        },
    );

    journals.reverse();
    for (i, journal_text) in journals.iter().enumerate() {
        let entry_count = if page_num == 0 && i == (journals.len() - 1) {
            String::from("")
        } else {
            format!(" - {}", (start as usize) + journals.len() - 1 - i)
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
    fn newlines_to_nl_tab(content: &mut String, newlines: usize) {
        let mut new_content = String::from("");
        new_content.push_str(&content[0..newlines]);
        new_content.push_str("\n\t");
        new_content.push_str(&content[(newlines + 2)..]);
        content.clear();
        content.push_str(&new_content[..]);
    }

    fn nl_tab_to_newlines(content: &mut String, nl_tab: usize) {
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
