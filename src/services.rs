use std::time::Duration;

use tokio::sync::mpsc;

use crate::http_tester::{HttpDownloadMeasurement, HttpLatencyMeasurement, HttpTester, HttpUploadMeasurement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpTestState {
    Idle,
    MeasuringLatency,
    MeasuringDownload,
    MeasuringUpload,
    Finished,
}

pub struct HttpTestService {
    tester: HttpTester,
    ping_test: HttpLatencyMeasurement,
    download_test: HttpDownloadMeasurement,
    upload_test: HttpUploadMeasurement,
    state: HttpTestState,
    ping_rx: Option<mpsc::UnboundedReceiver<HttpLatencyMeasurement>>,
    download_rx: Option<mpsc::UnboundedReceiver<HttpDownloadMeasurement>>,
    upload_rx: Option<mpsc::UnboundedReceiver<HttpUploadMeasurement>>,
}

impl HttpTestService {
    pub fn new(tester: HttpTester) -> Self {
        HttpTestService {
            tester,
            ping_test: HttpLatencyMeasurement::default(),
            download_test: HttpDownloadMeasurement::default(),
            upload_test: HttpUploadMeasurement::default(),
            state: HttpTestState::Idle,
            ping_rx: Some(mpsc::unbounded_channel().1),
            download_rx: Some(mpsc::unbounded_channel().1),
            upload_rx: Some(mpsc::unbounded_channel().1),
        }
    }

    pub fn set_tester(&mut self, tester: HttpTester) {
        self.tester = tester;
    }

    pub fn get_ping_results(&self) -> &HttpLatencyMeasurement {
        &self.ping_test
    }

    pub fn get_download_results(&self) -> &HttpDownloadMeasurement {
        &self.download_test
    }

    pub fn get_upload_results(&self) -> &HttpUploadMeasurement {
        &self.upload_test
    }

    pub fn get_testing(&self) -> bool {
        self.state != HttpTestState::Idle 
    }

    pub fn get_state(&self) -> &HttpTestState {
        &self.state
    }

    pub fn run_full_test(&mut self) {
        if self.state != HttpTestState::Idle {
            return;
        }
        self.state = HttpTestState::MeasuringLatency;

        self.run_current_state();
    }

    pub fn run_current_state(&mut self) {
        match self.state {
            HttpTestState::MeasuringLatency => self.run_latency_test(),
            HttpTestState::MeasuringDownload => self.run_download_test(),
            HttpTestState::MeasuringUpload => self.run_upload_test(),
            _ => {
                self.state = HttpTestState::Idle;
            }
        }
    }

    pub async fn check_measurments(&mut self) {
        if let Some(ref mut rx) = self.ping_rx {
            if let Ok(latency) = rx.try_recv() {
                self.ping_test = latency;
                self.state = HttpTestState::MeasuringDownload;
                self.run_current_state();
                return ;
            }
        }
        if let Some(ref mut rx) = self.download_rx {
            if let Ok(download) = rx.try_recv() {
                self.download_test = download;
                self.state = HttpTestState::MeasuringUpload;
                self.run_current_state();
                return ;
            }
        }
        if let Some(ref mut rx) = self.upload_rx {
            if let Ok(upload) = rx.try_recv() {
                self.upload_test = upload;
                self.state = HttpTestState::Finished;
                return ;
            }
        }
        if self.state == HttpTestState::Finished {
            self.state = HttpTestState::Idle;
        }
    }

    pub fn run_latency_test(&mut self) {
        self.state = HttpTestState::MeasuringLatency;
        let tester = self.tester.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.ping_rx = Some(rx);
        tokio::spawn(async move {
            let latency = tester.measure_latency_multiple(20).await;
            match latency {
                Ok(latency) => {
                    if tx.send(latency).is_err() {
                        let _ = tx.send(HttpLatencyMeasurement::default());
                    }
                }
                Err(_) => {
                    let _ = tx.send(HttpLatencyMeasurement::default());
                }
            }
        });
    }

    pub fn run_download_test(&mut self) {
        self.state = HttpTestState::MeasuringDownload;
        let tester = self.tester.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.download_rx = Some(rx);
        tokio::spawn(async move {
            let download = tester.measure_download().await;
            match download {
                Ok(download) => {
                    if tx.send(download).is_err() {
                        let _ = tx.send(HttpDownloadMeasurement::default());
                    }
                }
                Err(_) => {
                    let _ = tx.send(HttpDownloadMeasurement::default());
                }
            }
            
        });
    }

    pub fn run_upload_test(&mut self) {
        self.state = HttpTestState::MeasuringUpload;
        let tester = self.tester.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.upload_rx = Some(rx);
        tokio::spawn(async move {
            let response = tester.measure_upload().await;
            match response {
                Ok(upload) => {
                    if tx.send(upload).is_err() {
                        let _ = tx.send(HttpUploadMeasurement::default());
                    }
                }
                Err(_) => {
                    let _ = tx.send(HttpUploadMeasurement::default());
                }
            }
        });
    }
}