use serde_json::Value;
use std::fs::{OpenOptions, File};
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn save_weather_to_file(city: &str, weather_data: &Value) -> io::Result<()> {
    let timestamp = get_current_timestamp();
    let mut history: Vec<Value> = load_weather_history()?; // Load previous weather history

    let entry = serde_json::json!({
        "city": city,
        "timestamp": timestamp,
        "data": weather_data,
    });

    history.push(entry); // Append new entry

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("weather_history.json")?;
    
    serde_json::to_writer(file, &history)?; // Save updated history

    Ok(())
}
pub fn load_weather_history() -> io::Result<Vec<Value>> {
    let file = File::open("weather_history.json");

    match file {
        Ok(mut f) => {
            let mut data = String::new();
            f.read_to_string(&mut data)?;
            if data.trim().is_empty() {
                Ok(Vec::new()) // Return empty history if file is empty
            } else {
                let history: Vec<Value> = serde_json::from_str(&data)?;
                Ok(history)
            }
        }
        Err(_) => Ok(Vec::new()), // Return empty history if file doesn't exist
    }
}

pub fn clean_old_data() -> io::Result<()> {
    let history: Vec<Value> = load_weather_history()?;  //Loading current weather history
    let current_timestamp = get_current_timestamp();  //Get current time

// To filter entries and keep only those within 48hours

let filtered_history: Vec<Value> = history
    .into_iter()
    .filter(|entry|{
        let entry_timestamp = entry["timestamp"].as_u64().unwrap();
        current_timestamp - entry_timestamp < 172_800 //48 hours
    })
    .collect();

    // save filtered data back
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("weather_history.json")?;
    serde_json::to_writer(file, &filtered_history)?;

    Ok(())

}