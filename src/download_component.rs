use ratatui::{style::Stylize, symbols::block, text::{Line, Text}, widgets::{Block, Widget}};

use crate::http_tester::HttpDownloadMeasurement;

#[derive(Default, Clone)]
pub struct DownloadComponent {
    download_measurement: HttpDownloadMeasurement,
}

impl DownloadComponent {
    pub fn new() -> Self {
        Self {
            download_measurement: HttpDownloadMeasurement::default(),
        }
    }

    pub fn set_download_measurement(&mut self, measurement: HttpDownloadMeasurement) {
        self.download_measurement = measurement;
    }
}

impl Widget for &DownloadComponent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from("Download Speed").bold();
        let content = Text::from(vec![
            Line::from(format!("Downloaded data: {} MB", self.download_measurement.bits / (1024 * 1024 * 8)).green()),
            Line::from(format!("Duration: {:.2} seconds", self.download_measurement.duration.as_secs_f64()).red()),
            Line::from(format!("Speed: {:.2} Mbps", self.download_measurement.speed / (1024 * 1024) as f64).blue()),
        ]);

        let block = Block::bordered()
            .title(title);
        
        let paragraph = ratatui::widgets::Paragraph::new(content)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        paragraph.render(area, buf);
    }
}