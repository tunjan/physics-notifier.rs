use std::{
    fs,
    path::PathBuf,
    thread,
    time::Duration,
};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use reqwest::blocking::Client;
use notify_rust::Notification;
use rand::{
    thread_rng,
    seq::SliceRandom,
};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    interval: Option<u64>,
}

struct FactFetcher {
    client: Client,
}

impl FactFetcher {
    fn new() -> Self {
        FactFetcher { client: Client::new() }
    }

    fn fetch_online_facts(&self, url: &str) -> Result<Vec<String>, reqwest::Error> {
        let response = self.client.get(url).send()?;
        if response.status().is_success() {
            Ok(response.text()?.lines().map(String::from).collect())
        } else {
            Err(reqwest::Error::from(response.error_for_status().unwrap_err()))
        }
    }
}

fn read_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = PathBuf::from("config.json");
    let config_str = fs::read_to_string(config_path)?;
    Ok(from_str(&config_str)?)
}

fn read_local_facts(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    let contents = fs::read_to_string(file_path)?;
    Ok(contents.lines().map(String::from).filter(|line| !line.is_empty()).collect())
}

fn send_notification(fact: &str) {
    Notification::new()
        .summary("Interesting Physics Fact")
        .body(fact)
        .appname("PhysicsFacts")
        .timeout(10000)
        .show()
        .unwrap(); // Consider handling this error as well
}

fn main() {
    let config = read_config().expect("Failed to load configuration");
    let interval = config.interval.unwrap_or(15);
    let fact_fetcher = FactFetcher::new();
    let physics_facts = fact_fetcher.fetch_online_facts("https://some-url.com/facts")
        .or_else(|_| read_local_facts("physics_facts.txt"))
        .expect("Failed to fetch facts");

    let mut rng = thread_rng();
    
    loop {
        if let Some(fact) = physics_facts.choose(&mut rng) {
            send_notification(fact);
            println!("Sent fact: {}", fact);
        }
        thread::sleep(Duration::from_secs(interval));
    }
}
