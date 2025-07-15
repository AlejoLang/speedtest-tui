use reqwest::{Client};
use std::{io::Error, time::{Duration, Instant}};
pub enum HttpDownloadSize {
    S250,
    S350,
    S500,
    S750,
    S1000,
    S1500,
    S2000,
    S2500,
    S3000,
    S3500,
    S4000,
}

impl HttpDownloadSize {
    pub fn to_size(&self) -> usize {
        match self {
            HttpDownloadSize::S250 => 250,
            HttpDownloadSize::S350 => 350,
            HttpDownloadSize::S500 => 500,
            HttpDownloadSize::S750 => 750,
            HttpDownloadSize::S1000 => 1000,
            HttpDownloadSize::S1500 => 1500,
            HttpDownloadSize::S2000 => 2000,
            HttpDownloadSize::S2500 => 2500,
            HttpDownloadSize::S3000 => 3000,
            HttpDownloadSize::S3500 => 3500,
            HttpDownloadSize::S4000 => 4000,
        }
    }
    pub fn all() -> Vec<usize> {
        vec![
            HttpDownloadSize::S250.to_size(),
            HttpDownloadSize::S350.to_size(),
            HttpDownloadSize::S500.to_size(),
            HttpDownloadSize::S750.to_size(),
            HttpDownloadSize::S1000.to_size(),
            HttpDownloadSize::S1500.to_size(),
            HttpDownloadSize::S2000.to_size(),
            HttpDownloadSize::S2500.to_size(),
            HttpDownloadSize::S3000.to_size(),
            HttpDownloadSize::S3500.to_size(),
            HttpDownloadSize::S4000.to_size(),
        ]
    }
    pub fn min() -> HttpDownloadSize {
        HttpDownloadSize::S250
    }
    pub fn max() -> HttpDownloadSize {
        HttpDownloadSize::S2000
    }
}

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
                    min = min.min(latency);
                    max = max.max(latency);
                    latency_total += latency;
                    total_measurments += 1;
                }
                Err(e) => {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Latency measurement error: {}", e)))?;
                },
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await; // Sleep to avoid overwhelming the server
        }

        let avg = latency_total / total_measurments as f64;
        Ok(HttpLatencyMeasurement { min, max, avg, total_measurments })
    }

    pub async fn measure_download(&self) -> Result<HttpDownloadMeasurement, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; speedtest-tui/1.0)")
            .build()
            .expect("Failed to build Client");
        let url = self.url.clone() + format!("/speedtest/random{}x{}.jpg", HttpDownloadSize::S2000.to_size(), HttpDownloadSize::S2000.to_size()).as_str();

        let start = Instant::now();
        let response = client.get(url.as_str()).send().await;
        
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let content_length = resp.content_length().unwrap_or(0);
                    let duration = start.elapsed();
                    let bits = content_length * 8; // Convert bytes to bits
                    let speed = bits as f64 / duration.as_secs_f64(); // bits per second
                    return Ok(HttpDownloadMeasurement { bits, duration, speed })
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

    pub async fn measure_upload(&self) -> Result<HttpUploadMeasurement, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; speedtest-tui/1.0)")
            .build()
            .expect("Failed to build Client");
        
        let url = self.url.clone() + "/speedtest/upload.php";
        let data = vec![0u8; 10 * 1024 * 1024]; // 1 MB of data
        let start = Instant::now();
        
        let response = client.post(url.as_str())
            .body(data.clone())
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let duration = start.elapsed();
                    let bits = (data.len() * 8) as u64;
                    let speed = bits as f64 / duration.as_secs_f64();
                    return Ok(HttpUploadMeasurement { bits, duration, speed })
                } else {
                    Err(Error::new(std::io::ErrorKind::Other, "Request failed"))
                }
            }
            Err(e) => {
                Err(Error::new(std::io::ErrorKind::Other, format!("Request error: {}", e)))
            }
        }
    }
}
