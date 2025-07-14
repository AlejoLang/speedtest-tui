use crate::http_tester::{HttpLatencyMeasurement, HttpTester};
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Block, Widget}};

#[derive(Default, Clone)]
pub struct PingComponent {
    ping_measurement: HttpLatencyMeasurement,
}

impl PingComponent {
    pub fn set_ping_measurement(&mut self, ping: HttpLatencyMeasurement) {
        self.ping_measurement = ping;
    }
}

impl Widget for &PingComponent{
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
         let ping_res = Text::from(vec![
            Line::from(format!("Average Latency: {:.2} ms", self.ping_measurement.avg))
                .bold()
                .blue()
                .centered(),
            Line::from(format!("Min Latency: {:.2} ms", self.ping_measurement.min))
                .bold()
                .green()
                .centered(),
            Line::from(format!("Max Latency: {:.2} ms", self.ping_measurement.max))
                .bold()
                .red()
                .centered(),
        ]);

        let block = Block::bordered()
            .title("Ping Component")
            .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
            .title_style(ratatui::style::Style::default().fg(ratatui::style::Color::White).bold());

        let paragraph = ratatui::widgets::Paragraph::new(ping_res)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        paragraph.render(area, buf); 
    }
}
