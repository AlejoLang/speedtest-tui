use std::net::TcpStream;
use std::time::Instant;

#[derive(Clone, Default)]
pub struct TcpTest {
    host: String,
    port: u16,
}

#[derive(Default, Clone)]
pub struct TcpTestLatency {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
}

impl TcpTest {
    pub fn new(host: String, port: u16) -> Self {
        TcpTest { host, port }
    }

    pub fn measure_latency(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let _stream = TcpStream::connect(self.host.as_str())?;
        let duration = start.elapsed();
        Ok(duration.as_millis() as f64)
    }

    pub async fn measure_latency_multiple(&self, count: usize) -> Result<TcpTestLatency, Box<dyn std::error::Error>> {
        let mut latency_test = TcpTestLatency::default();
        latency_test.min = f64::MAX;
        latency_test.max = f64::MIN;
        let mut total_latency = 0.0;
        for _ in 0..count {
            let new_test = self.measure_latency()?;
            latency_test.min = latency_test.min.min(new_test);
            latency_test.max = latency_test.max.max(new_test);
            total_latency += new_test;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await; // Sleep to avoid overwhelming the server
        }
        latency_test.avg = total_latency / count as f64;
        Ok(latency_test)
    }
}
