use ratatui::{style::{Color, Style, Stylize}, text::{Line, Text}, widgets::{Block, Paragraph, Widget}};

use crate::http_tester::HttpUploadMeasurement;

#[derive(Default, Clone)]
pub struct UploadComponent {
    upload_measurement: HttpUploadMeasurement,
    active: bool,
}

impl UploadComponent {
    pub fn set_upload_measurement(&mut self, measurement: HttpUploadMeasurement) {
        self.upload_measurement = measurement;
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Widget for &UploadComponent {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from("Upload Speed").bold();
        let content = Text::from(vec![
            Line::from(format!("Uploaded data: {} MB", self.upload_measurement.bits / (1024 * 1024 * 8)).green()),
            Line::from(format!("Duration: {:.2} seconds", self.upload_measurement.duration.as_secs_f64()).red()),
            Line::from(format!("Speed: {:.2} Mbps", self.upload_measurement.speed / (1024 * 1024) as f64).blue()),
        ]);

        let block = Block::bordered()
            .border_style(Style::default().fg(if self.active {Color::Green } else { Color::Red }))
            .title(title);
        
        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        paragraph.render(area, buf);
    }
}