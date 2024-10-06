use clap::Parser;
use reqwest::blocking::get;
use serde_json::Value;
use std::error::Error;
use std::collections::HashMap;

mod history;
#[derive(Parser)]
struct Args {
    /// City name (optional). Auto-detect location if not provided.
    city: Option<String>,

    /// Units for temperature (metric or imperial)
    #[arg(short, long, default_value = "metric")]
    units: String,
}

fn fetch_weather(city: &str, api_key: &str, units: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/forecast?q={city}&appid={api_key}&units={units}"
    );
    let response = get(&url)?.json::<Value>()?;
    Ok(response)
}

fn get_user_location() -> Result<String, Box<dyn Error>> {
    let location_url = "https://ipinfo.io/json";
    let response = get(location_url)?.json::<Value>()?;
    if let Some(city) = response["city"].as_str() {
        Ok(city.to_string())
    } else {
        Err("Could not detect location.".into())
    }
}

fn group_forecast_by_day(forecast: &Value) -> HashMap<String, Vec<f64>> {
    let mut daily_data: HashMap<String, Vec<f64>> = HashMap::new();
    
    if let Some(list) = forecast["list"].as_array() {
        for entry in list {
            if let (Some(dt_txt), Some(temp)) = (entry["dt_txt"].as_str(), entry["main"]["temp"].as_f64()) {
                let date = &dt_txt[..10]; // Extract the date (YYYY-MM-DD)
                daily_data.entry(date.to_string()).or_insert(vec![]).push(temp);
            }
        }
    }

    daily_data
}

fn summarize_daily_temps(daily_data: HashMap<String, Vec<f64>>) {
    for (date, temps) in daily_data {
        let avg_temp = temps.iter().sum::<f64>() / temps.len() as f64;
        let min_temp = temps.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_temp = temps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        println!("Date: {}, Avg: {:.2}°C, Min: {:.2}°C, Max: {:.2}°C", date, avg_temp, min_temp, max_temp);
    }
}

fn main() {
    let args = Args::parse();

    //Clean old history entriesbefore new fetch
    history::clean_old_data().expect("Failed to clean old data");

    let city = match &args.city {
        Some(city) => city.clone(),
        None => match get_user_location() {
            Ok(detected_city) => {
                println!("Auto-detected location: {}", detected_city);
                detected_city
            }
            Err(e) => {
                eprintln!("Error detecting location: {}", e);
                return;
            }
        },
    };

    let api_key = "c9f83bbf47865db6323f9b4fb45ddff9";
    match fetch_weather(&city, api_key, &args.units) {
        Ok(forecast_data) => {
            //save waether data
            history::save_weather_to_file(&city, &forecast_data).expect("Failed to save weather data");
            let daily_data = group_forecast_by_day(&forecast_data);
            summarize_daily_temps(daily_data);
        }
        Err(e) => eprintln!("Error fetching weather: {}", e),
    }
}
