use std::env;
use std::collections::HashMap;
use std::{fs, io::{self, BufRead}};
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use std::fmt;
use std::process::Command;

#[derive(Debug)]
struct MyError {
    details: String
}

impl MyError {
    fn new(msg: &str) -> MyError {
        MyError{details: msg.to_string()}
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        &self.details
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_name = get_project_name()?;
    if !Path::new(".tickets_config").exists() {
        print!("Config file not found. Would you like to create one? [Y/n]: ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        match user_input.trim().to_lowercase().as_str() {
            "y" => create_config(&default_name)?,
            _ => return Ok(())
        };
    }
    let mut args = env::args();
    // get first arg from list in run_command
    let command = get_command(&mut args)?;
    run_command(&command, &mut args)?;
    Ok(())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Vec<String>, &'static str> {
    let mut parsed_args: Vec<String> = Vec::new();
    let valid_args = std::collections::HashSet::from([
        "--closed",
        "--open",
        "--current",
        "--complete"
    ]);
    for arg in args {
        if valid_args.contains(arg.trim()) {
            parsed_args.push(arg[2..].to_string());
        }
        
    }
    Ok(parsed_args)
}

fn search_file(contents: &str, parsed_args: &Vec<String>) -> bool {
    if parsed_args.is_empty() {
        return true
    }
    let mut configs: HashMap<&str, &str> = HashMap::new();
    let mut lines = contents.lines();
    while let Some(line) = lines.next() {
        if line.contains("=============") {
            break;
        }
        let (key, value) = match line.split_once(':') {
            Some((k, v)) => (k, v),
            None => ("", ""),
        };
        configs.insert(key, value);
    }

    for status in parsed_args {
        if configs.get("status").unwrap() == status {
            return true
        }
    }
    false
}

fn get_command(mut args: impl Iterator<Item = String>) -> Result<String, &'static str> {
    // skip executable
    args.next();

    match args.next() {
        Some(arg) => Ok(arg),
        None => return Err("No action found")
    }
}

fn run_command(command: &str, args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        "list" => list_tickets(args, "")?,
        "current" => list_tickets(args, "in_progess")?,
        "new" => new_ticket(args)?,
        "close" => edit_tickets_status(args, "closed")?,
        "open" => edit_tickets_status(args, "open")?,
        "complete" => edit_tickets_status(args, "complete")?,
        "start" => edit_tickets_status(args, "in_progress")?,
        "edit" => edit_ticket(args)?,
        "help" => print_help(),
        _ => return Err(Box::new(MyError::new(format!("Unrecognized command: {}", command).as_str())))
    }
    Ok(())
}

fn list_tickets(args: impl Iterator<Item = String>, status: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_args = parse_args(args)?;
    let config = get_config()?;
    let project_tickets_path = format!("{}/{}", ticket_path()?, config.get("project_name").unwrap_or(&get_project_name()?));
    fs::create_dir_all(&project_tickets_path)?;
    let entries = fs::read_dir(&project_tickets_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    for entry in entries {
        let contents = fs::read_to_string(entry.to_str().unwrap())?;
        if search_file(&contents, &parsed_args) {
            println!("{}", contents);
        }
    }
    Ok(())
}

fn get_next_file_name(project_tickets_path: &str) -> Result<String, Box<dyn Error>> {
    fs::create_dir_all(&project_tickets_path)?;
    let mut entries = fs::read_dir(&project_tickets_path)?
        .map(|res|
             res.map(|e| e.file_name().to_string_lossy().to_string().parse::<u32>().expect("failed to parse filename while getting next file name") )
         )
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();

    if entries.len() == 0 {
        return Ok("0".to_string());
    }
    println!("entries: {:?}", entries);
    let new_filename = match entries.last() {
        Some(last_ticket) => last_ticket + 1,
        None => return Err(Box::new(MyError::new("borked"))),
    };
    Ok(new_filename.to_string())
}

fn new_ticket(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let config = get_config()?;
    let project_tickets_path = format!("{}/{}", ticket_path()?, config.get("project_name").unwrap_or(&get_project_name()?));
    let num = get_next_file_name(&project_tickets_path)?;
    let mut file = fs::File::create(format!("{}/{}", project_tickets_path, num))?;
    // TODO: look for a replacement for new lines. Something like writeln!
    let template = format!("ticket:{}\nstatus:open\nowner:jin\n================\n{}\n\n", num, get_content(&mut args)).to_string();
    file.write_all(&template.as_bytes())?;
    Ok(())
}

fn edit_tickets_status(args: impl Iterator<Item = String>, ticket_status: &str) -> Result<(), Box<dyn Error>> {
    let config = get_config()?;
    let project_tickets_path = format!("{}/{}", ticket_path()?, config.get("project_name").unwrap_or(&get_project_name()?));
    let files: Vec<String> = args.map(|arg| format!("{}/{}", project_tickets_path, arg)).collect();
    let output = Command::new("sed")
        .arg("-i")
        .arg(format!("s/^status:\\S*$/status:{}/", ticket_status))
        .args(files)
        .spawn()?;
    println!("edit_tickets_status output: {:#?}", output);
    Ok(())
}

fn edit_ticket(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let config = get_config()?;
    let project_tickets_path = format!("{}/{}", ticket_path()?, config.get("project_name").unwrap_or(&get_project_name()?));
    let file = match args.next() {
        Some(arg) => format!("{}/{}", project_tickets_path, arg),
        None => return Err(Box::new(MyError::new("No filename found")))
    };
    let output = Command::new("vim")
        .arg(file)
        .status()?;
    println!("edit_ticket output: {:#?}", output);
    Ok(())
}

fn print_help() {
    println!("Tickets will create a .tickets_config file that will hold a project_name, which will be the directory name");
    println!("of the project in the ${{HOME}}/{{project_name}}/ directory.");
    println!("Commands you can run include:");
    println!("list                      Lists tickets for this project");
    println!("new                       Creates a new ticket");
    println!("close                     Marks a ticket as closed");
    println!("open                      Marks a ticket as open");
    println!("complete                  Marks a ticket as complete");
    println!("start                     Marks a ticket as in_progress");
    println!("edit                      Opens a ticket in vim");
    println!("help                      Print this menu");
}

fn create_config(default_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut user_input = String::new();
    print!("Enter a name for the project (default: {default_name}): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user_input)?;
    user_input = user_input.split_whitespace().next().unwrap_or(default_name).trim().to_string();
    let mut f = fs::File::create(".tickets_config")?;
    writeln!(f, "project_name:{user_input}")?;
    Ok(())
}

fn get_config() -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut config: HashMap<String, String> = HashMap::new();
    if let Ok(lines) = read_lines(".tickets_config") {
        for line in lines {
            if let Ok(s) = line {
                // set config map
                let (k, v) = s.split_once(":").unwrap();
                config.insert(k.to_string(), v.to_string());
            }
        }
    }
    Ok(config)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where P: AsRef<Path>, {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_project_name() -> Result<String, &'static str> {
    let current_dir = match env::current_dir() {
        Ok(curdur) => curdur,
        Err(_err) => return Err("Unable to get current directory"),
    };
    let project_name = match current_dir.file_name() {
        Some(name) => name,
        None => return Err("Unable to get project name"),
    };
    Ok(project_name.to_string_lossy().to_string())
}

fn get_content(mut args: impl Iterator<Item = String>) -> String {
    args.next().unwrap_or("Ticket description should go here".to_string())
}

fn ticket_path() -> Result<String, &'static str> {
    let mut home_path = match home::home_dir() {
        Some(path) => path,
        None => return Err("Did not find a home directory"),
    };
    home_path.push(".tickets");
    Ok(home_path.to_string_lossy().to_string())
}

