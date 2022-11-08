use std::env;
use std::process;
use std::{fs, io};
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_path = ticket_path().unwrap_or_else(|err| {
        eprintln!("error: {err}");
        process::exit(1);
    });
    let content = content(env::args());
    fs::create_dir_all(&target_path)?;

    let mut entries = fs::read_dir(&target_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();
    let name = get_next_file_name(&entries)?;

    let mut file = fs::File::create(format!("{}/{}", target_path, name))?;
    let template = format!("ticket:{}\nresponsible:jin\nstatus:open\n================\n{}\n", name, content).to_string();
    file.write_all(&template.as_bytes())?;

    Ok(())
}

fn content(mut args: impl Iterator<Item = String>) -> String {
    // skip executable
    args.next();

    let file_path = args.next().unwrap_or("".to_string());
    file_path
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
    let current_dir = match env::current_dir() {
        Ok(curdur) => curdur,
        Err(_err) => return Err("Unable to get current directory"),
    };
    let project_name = match current_dir.file_name() {
        Some(name) => name,
        None => return Err("Unable to get project name"),
    };

    home_path.push(".tickets");
    home_path.push(&project_name);
    Ok(home_path.to_string_lossy().to_string())
}

