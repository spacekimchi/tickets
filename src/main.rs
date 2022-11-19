use std::env;
use std::collections::HashMap;
use std::{fs, io::{self, BufRead}};
use std::io::prelude::*;
use std::path::{PathBuf, Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_name = get_project_name()?;
    if !Path::new(".tickets_config").exists() {
        print!("Config file not found. Would you like to create one? [Y/n]: ");
        io::stdout().flush().unwrap();
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        match user_input.as_str().trim() {
            "Y" => create_config(&default_name)?,
            "y" => create_config(&default_name)?,
            _ => return Ok(())
        };
    }

    let ticket_path = ticket_path()?; /* $HOME/.tickets/ */
    let mut config = HashMap::new();
    get_config(&default_name, &ticket_path, &mut config)?;

    let content = content(env::args()); /* content body for the ticket */
    let project_tickets_path = format!("{}/{}", ticket_path, config.get("project_name").unwrap_or(&default_name));
    fs::create_dir_all(&project_tickets_path)?;
    let mut entries = fs::read_dir(&project_tickets_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    let num = get_next_file_name(&entries)?;
    let mut file = fs::File::create(format!("{}/{}", project_tickets_path, num))?;
    let template = format!("ticket:{}\nstatus:open\n================\n{}\n\n", num, content).to_string();
    file.write_all(&template.as_bytes())?;
    Ok(())
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

fn get_config(default_name: &str, ticket_path: &str, config: &mut HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(".tickets_config").exists() {
        let mut user_input = String::new();
        while user_input.len() == 0 {
            print!("Enter a name for the project(no spaces please): ");
            io::stdin().read_line(&mut user_input)?;
            user_input = user_input.split_whitespace().next().unwrap_or("").trim().to_string();
        }
        let mut f = fs::File::create(".tickets_config")?;
        write!(f, "project_name:{user_input}")?;
        config.insert("project_name".to_string(), user_input.to_string());
        return Ok(());
    }
    if let Ok(lines) = read_lines(".tickets_config") {
        for line in lines {
            if let Ok(s) = line {
                // set config map
                let (k, v) = s.split_once(":").unwrap();
                config.insert(k.to_string(), v.to_string());
            }
        }
    }

    Ok(())
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

fn content(mut args: impl Iterator<Item = String>) -> String {
    // skip executable
    args.next();
    args.next().unwrap_or("".to_string())
}

fn get_next_file_name(dirs: &Vec<PathBuf>) -> Result<String, &'static str> {
    if dirs.len() == 0 {
        return Ok("0".to_string());
    }
    let last_ticket = match dirs.last() {
        Some(num) => num.file_name(),
        None => return Err("Unable to get next file name"),
    };
    let last_ticket_number = match last_ticket {
        Some(val) => val.to_string_lossy().to_string(),
        None => "0".to_string(),
    };
    let name: u32 = last_ticket_number.parse().unwrap_or(0) + 1;
    Ok(name.to_string())
}

fn ticket_path() -> Result<String, &'static str> {
    let mut home_path = match home::home_dir() {
        Some(path) => path,
        None => return Err("Did not find a home directory"),
    };
    home_path.push(".tickets");
    Ok(home_path.to_string_lossy().to_string())
}

