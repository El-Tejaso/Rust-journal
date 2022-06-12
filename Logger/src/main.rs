use chrono::{self, Date, Datelike, Duration, Timelike, TimeZone};
use chrono::{DateTime, Local};
use console::Term;
use regex::Regex;
use std::env;
use std::ffi::OsStr;
use std::fs::{self};
use std::io::{self, ErrorKind, Write};
use std::path::PathBuf;
use lazy_static::lazy_static;

fn now() -> DateTime<Local> {
    Local::now()
}

lazy_static! {
    static ref DATE_RE: Regex = Regex::new(r"(\d{4})/(\d{1,2})/(\d{1,2})").unwrap();
}

fn datestamp(time: &DateTime<Local>) -> String {
    format!(
        "{}/{}/{}",
        time.year(),
        &two_dig_number(time.month()),
        &two_dig_number(time.day())
    )
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

fn write_log(path: &PathBuf, contents: &str) {
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

fn logging_line(date: &DateTime<Local>, indent: usize, contents: &str) -> String {
    let mut line = String::from("\n");

    for _i in 0..indent {
        line.push_str("\t");
    }

    line.push_str(&timestamp(&date));
    line.push_str(" - ");
    line.push_str(&contents);

    return line;
}

fn new_log_text(name: &OsStr, date: &DateTime<Local>) -> String {
    let ds = datestamp(date);
    format!("{} - {}\n", name.to_string_lossy(), ds)
}

fn load_log(path: &PathBuf) -> String {
    return match read_file(path) {
        Ok(str) => str.replace("\r", ""),
        Err(e) => {
            panic!("Couldn't read the contents of {}: {}", &path.display(), e);
        }
    };
}

fn get_input_str() -> String {
    let stdin = io::stdin();

    let mut input = String::from("");
    if let Err(_) = stdin.read_line(&mut input) {
        return String::from("");
    }

    input = input.trim_end().to_string();

    if input.starts_with(":") || input.starts_with("/") {
        let command = &input[1..];
        if command == "exit" {
            clear_screen();
            std::process::exit(0);
        } else if command == "help" {
            print_help();
            return String::from("");
        }
    }

    return input;
}

fn display_log(path: &PathBuf) {
    let content = load_log(path);
    println!("{}", &content);
}

fn clear_screen() {
    let term = Term::stdout();
    term.clear_screen().expect("failed clearing screen");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    // WHY, RUST, WHY ??
    let exe_name = format!(
        "{}",
        env::current_exe()
            .expect("must be an exe>WTF")
            .file_name()
            .expect("file must have a name")
            .to_string_lossy()
    );
    let name = &exe_name;

    if args.len() < 2 {
        println!("usage: {} [filepath]", name);
        return;
    }

    let mut path = PathBuf::from(args[1].clone());
    if path.extension() == None {
        path.set_extension("txt");
    }

    match read_file(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                let name = path.file_name().expect("file must have a name");

                let content = new_log_text(name, &now());
                write_log(&path, &content);
            }
        }
        _ => {}
    }

    loop {
        clear_screen();
        display_log(&path);

        print!("\n$~{}\n> ", &path.display());
        if let Err(_) = io::stdout().flush() {}

        let input = get_input_str();

        // process input
        if input.trim() == "" || input.trim() == "-" {
            continue;
        } else if input.trim().starts_with("/") || input.trim().starts_with(":") {
            let command = &input.trim()[1..];
            if command.starts_with("time") {
                display_time_stats(&path, false);
            } else if command.starts_with("gtime") {
                display_time_stats(&path, true);
            }

            continue;
        } else if input.starts_with("help") {
            continue;
        }

        match append_to_log(&path, input) {
            Ok(new_content) => {
                write_log(&path, &new_content);
            }
            Err(e) => {
                panic!("what happened!?!?\n\n{}", e);
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

09:54 am - There is no way to delete logs, and I don't want to add this because I want the contents to be more or less unfiltered
"
    );

    println!("Press enter to continue...");
    get_input_str();
}

fn display_time_stats(path: &PathBuf, granular: bool) {
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

    fn duration_to_str(duration: &Duration) -> String {
        let minutes = duration.num_minutes();
        let hours = duration.num_hours();
        let days = duration.num_days();

        let mut str = format!("");

        if hours==0 && minutes==0 && days==0 {
            return format!("0");
        }

        if days > 0 {
            str += &format!("{}d ", days);

            if hours > 0 || minutes > 0 {
                str += &format!(" ");
            }
        }

        if hours > 0 {
            str += &format!("{:.2}h", hours % 24);

            if minutes > 0 {
                str += &format!(" ");
            }
        }

        if minutes > 0 {
            str += &format!("{}m", minutes % 60);
        }

        return str;
    }

    let text = load_log(path);
    let mut times: Vec<(DateTime<Local>, &str, bool)> = Vec::new();

    let mut current_date_opt: Option<DateTime<Local>> = None;
    let mut new_date = false;
    for line in text.split("\n") {
        let mut is_block = line.find("\t") == None;

        if let Some(parsed_date) = parse_date(line) {
            current_date_opt = Some(parsed_date.and_hms(0, 0, 0));
            new_date = true;
        }
        
        if let Some(current_date) = current_date_opt {
            if let Some(time) = parse_time(line, &current_date) {
                if new_date {
                    new_date = false;
                    is_block = true;
                }

                times.push((time, line, is_block));
            }
        }
    }

    times.push((now(), "<now>", true));

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
            let is_block = times[i].2;
            if is_block {
                println!("");
            }

            if is_block || granular {
                let dt = times[i].0.signed_duration_since(times[i - 1].0);
                let dt_from_start = times[i].0.signed_duration_since(times[0].0);
                let dt_from_block = times[i].0.signed_duration_since(block_time);

                println!(
                    "\nelapsed:\t\tsince start: {}      since block: {}      since last: {}\n",
                    duration_to_str(&dt_from_start),
                    duration_to_str(&dt_from_block),
                    duration_to_str(&dt)
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

fn parse_date(line : &str) -> Option<Date<Local>> {
    for cap in DATE_RE.captures_iter(line) {
        let y = (&cap[1]).parse::<i32>().unwrap();
        let m = (&cap[2]).parse::<u32>().unwrap();
        let d = (&cap[3]).parse::<u32>().unwrap();

        return Some(Local.ymd(y, m, d));
    }

    return None;
}

fn append_to_log(path: &PathBuf, input: String) -> Result<String, String> {
    let mut content = load_log(path);
    let has_no_entries = content.matches("-").count() < 2;
    let date = now();

    // append a line containing today's date if it doesn't already exist, or if the date there is old.
    let mut is_new_day: bool = false;
    'outer: for line in content.rsplit("\n") {
        if line.trim().len() == 0 || line.contains("am") || line.contains("pm") {
            continue;
        }

        if let Some(date) = parse_date(line) {
            let y = date.year();
            let m = date.month();
            let d = date.day();

            if y < date.year() || m < date.month() || d < date.day() {
                is_new_day = true;
            } else {
                is_new_day = false;
            }
    
            break 'outer;
        }
    }

    if is_new_day {
        content.push_str(&format!("\n\n----\tDate: {}\t----\n\n", datestamp(&date)));
    }

    // append actual content

    if input.trim() == "~" {
        if !has_no_entries {
            toggle_block(&mut content);
        } else {
            return Err(String::from("Can't use '~' when there aren't any entries"));
        }
    } else if input.starts_with("-") || has_no_entries {
        let mut input = &input[..];
        if input.starts_with("-") {
            input = &input[1..];
        }

        let new_line = logging_line(&date, 0, &input);
        push_block(&mut content, &new_line);
    } else {
        // don't trim lines, only headings
        let new_line = logging_line(&date, 1, &input);
        push_line(&mut content, &new_line);
    }

    Ok(content)
}

fn push_block(content: &mut String, new_line: &str) {
    content.push_str("\n");
    content.push_str(&new_line);
}

fn push_line(content: &mut String, new_line: &str) {
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
