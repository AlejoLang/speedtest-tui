use reqwest::{Client};
use std::{io::Error, time::{Duration, Instant}};

#[derive(Debug, Default, Clone)]
pub struct HttpLatencyMeasurement {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub total_measurments: u8,
}

#[derive(Debug, Default, Clone)]
pub struct HttpUploadMeasurement {
    pub bits: u64,
    pub duration: Duration,
    pub speed: f64, // bits per second
}

#[derive(Debug, Default, Clone)]
pub struct HttpDownloadMeasurement {
    pub bits: u64,
    pub duration: Duration,
    pub speed: f64, // bits per second
}

#[derive(Debug, Default, Clone)]
pub struct HttpTester {
    pub url: String,
}

impl HttpTester {
    pub fn new(url: &str) -> Self {
        HttpTester {
            url: url.to_string(),
        }
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub async fn measure_latency(&self) -> Result<f64, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; speedtest-tui/1.0)")
            .build()
            .expect("Failed to build Client");
        let start = Instant::now();
        let response = client.head(self.url.as_str()).send().await;
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let duration = start.elapsed();
                    Ok(duration.as_millis() as f64)
                } else {
                    println!("{}: {}", resp.status(), resp.text().await.unwrap_or_default());
                    Err(Error::new(std::io::ErrorKind::Other, "Request failed"))
                }
            }
            Err(e) => {
                Err(Error::new(std::io::ErrorKind::Other, format!("Request error: {}", e)))
            }
        } 
    }

    pub async fn measure_latency_multiple(&self, count: u8) -> Result<HttpLatencyMeasurement, Error> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut latency_total = 0.0;
        let mut total_measurments: u8 = 0;

        for _ in 0..count {
            match self.measure_latency().await {
                Ok(latency) => {
                    println!("Latency: {:.2} ms", latency);
                    min = min.min(latency);
                    max = max.max(latency);
                    latency_total += latency;
                    total_measurments += 1;
                }
                Err(e) => {
                    println!("Error measuring latency: {}", e);
                },
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await; // Sleep to avoid overwhelming the server
        }

        let avg = latency_total / total_measurments as f64;
        Ok(HttpLatencyMeasurement { min, max, avg, total_measurments })
    }
    
}
