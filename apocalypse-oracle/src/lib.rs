#[allow(warnings)]

mod bindings;
use bindings::{Guest, Output, TaskQueueInput};

use chrono::Datelike;

use layer_wasi::{block_on, Reactor, Request, WasiPollable};

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

struct Component;

impl Guest for Component {
    fn run_task(_input: TaskQueueInput) -> Output {
        block_on(get_m2_supply)
    }
}

async fn get_m2_supply(reactor: Reactor) -> Result<Vec<u8>, String> {
    // Replace with your FRED API Key
    let api_key = "API_KEY_REPLACE";

    // Series IDs
    let m2_series_id = "M2SL"; // M2 Money Supply
    let cpi_series_id = "CPIAUCSL"; // Consumer Price Index for All Urban Consumers

    // Get the current date
    let today = chrono::Utc::today().naive_utc();
    let current_year = today.year();
    let current_month = today.month();

    // Calculate the previous month and year
    let (prev_year, prev_month) = if current_month == 1 {
        (current_year - 1, 12)
    } else {
        (current_year, current_month - 1)
    };

    // Define date strings
    let current_month_str = format!("{}-{:02}-01", current_year, current_month);
    let prev_month_str = format!("{}-{:02}-01", prev_year, prev_month);

    // Fetch M2 Money Supply data
    let m2_current = fetch_latest_value(&reactor, api_key, m2_series_id, &current_month_str).await?;

    Ok(m2_current.to_string().as_bytes().to_vec())
}

async fn fetch_latest_value(reactor: &Reactor, api_key: &str, series_id: &str, date: &str) -> Result<f32, String> {
    let url = "https://api.stlouisfed.org/fred/series/observations?series_id=M2SL&sort_order=desc&api_key=<API_KEY_REPLACE>&file_type=json"; 

    let mut req = Request::get(url)?;
    let res = &reactor.send(req).await?;

    let data: M2Supply = res.json::<M2Supply>()?;
    let cur_month_val = data.observations.first().unwrap().value.parse::<f32>().unwrap();
    let prev_month_val = data.observations[1].value.parse::<f32>().unwrap();

    let m2_change = calculate_percentage_change(prev_month_val as f64, cur_month_val as f64);

    Ok(m2_change.to_string().parse::<f32>().unwrap())
    
}

fn calculate_percentage_change(previous: f64, current: f64) -> f64 {
    if previous.abs() < std::f64::EPSILON {
        return 0.0;
    }
    ((current - previous) / previous) * 100.0
}

#[derive(Deserialize, Serialize, Debug)]
struct Observation {
  date: String, 
  realtime_end: String, 
  realtime_start: String, 
  value: String
}

#[derive(Deserialize, Serialize, Debug)]
struct M2Supply {
  pub count: u32,
  pub observations: Vec<Observation>
}

impl M2Supply {
    /// Serialize to JSON.
    fn to_json(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&self).map_err(|err| err.to_string())
    }
}

bindings::export!(Component with_types_in bindings);

