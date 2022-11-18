use std::env;
use std::collections::HashMap;
use std::{fs, io::{self, BufRead}};
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticket_path = ticket_path()?;
    let config = HashMap::new();
    let default_name = get_project_name()?;
    get_config(&default_name, &ticket_path, &config)?;
    let content = content(env::args());
    let project_tickets_path = format!("{}/{}", ticket_path, config.get("project_name").unwrap_or(&&default_name[..]));
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

fn get_config(default_name: &str, ticket_path: &str, config: &HashMap<&str, &str>) -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(".tickets_config").exists() {
        let mut user_input = String::new();
        while user_input.len() == 0 {
            print!("Enter a name for the project(no spaces please [default): ");
            io::stdin().read_line(&mut user_input)?;
            user_input = user_input.split_whitespace().next().unwrap_or("").trim().to_string();
        }
        let mut f = fs::File::create(".tickets_config")?;
        write!(f, "project_name:{user_input}");
        config.insert("project_name", &user_input.to_string());
        return Ok(());
    }
    let mut file = fs::File::open(".tickets_config")?;
    if let Ok(lines) = read_lines(".tickets_config") {
        for line in lines {
            if let Ok(s) = line {
                // set config map
                let (k, v) = s.split_once(":").unwrap();
                config.insert(k, v);
            }
        }
    }

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where P: AsRef<std::path::Path>, {
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

fn get_next_file_name(dirs: &Vec<std::path::PathBuf>) -> Result<String, &'static str> {
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

