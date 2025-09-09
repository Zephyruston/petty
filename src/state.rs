use crate::pet::Pet;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

fn get_state_path() -> io::Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or(io::Error::new(
        ErrorKind::NotFound,
        "Could not find home directory",
    ))?;
    path.push(".petty");
    fs::create_dir_all(&path)?;
    path.push("state.json");
    Ok(path)
}

pub fn save_state(pet: &Pet) -> io::Result<()> {
    let path = get_state_path()?;
    let data = serde_json::to_string(pet)?;
    fs::write(path, data)
}

pub fn load_state() -> io::Result<Pet> {
    let path = get_state_path()?;
    if !path.exists() {
        return Err(io::Error::new(ErrorKind::NotFound, "State file not found"));
    }
    let data = fs::read_to_string(path)?;
    let pet = serde_json::from_str(&data)?;
    Ok(pet)
}

pub fn delete_state() -> io::Result<()> {
    let path = get_state_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
