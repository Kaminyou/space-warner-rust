use std::{collections::HashMap, env, process::Command, thread, time::Duration};
use serde_json::json;
use log::error;
use reqwest::Client;

fn get_disk_usage() -> Vec<HashMap<String, String>> {
    let output = Command::new("df")
        .arg("-h")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.split('\n').collect();
    let headers: Vec<&str> = lines[0].split_whitespace().collect();

    let avail_index = headers.iter().position(|&x| x == "Avail").unwrap();
    let used_index = headers.iter().position(|&x| x == "Use%").unwrap();

    let mut data = vec![];

    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > used_index {
            let filesystem = parts[0].to_string();
            let avail = parts[avail_index].to_string();
            let used_percent = parts[used_index].to_string();

            let mut entry = HashMap::new();
            entry.insert("filesystem".to_string(), filesystem);
            entry.insert("avail".to_string(), avail);
            entry.insert("used%".to_string(), used_percent);

            data.push(entry);
        }
    }

    data
}

async fn warn(client: &Client, filesystem: &str, used: &str) {
    let api_endpoint = env::var("API_ENDPOINT").unwrap_or_default();
    let json_data = json!({
        "text": format!("WARNING: {}: used {}", filesystem, used),
    });

    if let Err(e) = client.post(&api_endpoint)
        .json(&json_data)
        .send()
        .await {
        error!("Error due to {:?}", e);
    }
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    let trigger_interval: u64 = env::var("TRIGGER_INTERVAL")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap();

    let warning_interval: u64 = env::var("WARNING_INTERVAL")
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap();

    loop {
        let disk_usage_info = get_disk_usage();
        let target_filesystems: Vec<String> = env::var("FILE_SYSTEMS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.to_string())
            .collect();
        let threshold: f32 = env::var("THRESHOLD").unwrap_or_else(|_| "1.0".to_string()).parse().unwrap();

        let mut is_warning = false;
        for info in disk_usage_info {
            if target_filesystems.contains(&info["filesystem"]) {
                let used = info["used%"].replace('%', "").parse::<f32>().unwrap();
                if used >= threshold {
                    warn(&client, &info["filesystem"], &info["used%"]).await;
                    is_warning = true;
                }
            }
        }

        if is_warning {
            thread::sleep(Duration::from_secs(warning_interval));
        } else {
            thread::sleep(Duration::from_secs(trigger_interval));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    use reqwest::Client;

    #[test]
    fn test_get_disk_usage() {
        let usage = get_disk_usage();
        assert!(!usage.is_empty()); // Basic check to ensure some data is returned
    }

    #[tokio::test]
    async fn test_warn() {
        let client = Client::new();
        let mock_server = mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("{\"status\":\"ok\"}")
            .create();

        // Set API_ENDPOINT to the mock server URL
        env::set_var("API_ENDPOINT", &server_url());

        // Call the warn function
        warn(&client, "testfs", "95%").await;

        // Check if the mock was called
        mock_server.assert();
    }
}
