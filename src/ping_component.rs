use crate::{servers::Server, tcp_tests::{TcpTest, TcpTestLatency}};
use ratatui::{style::Stylize, text::{Line, Text}, widgets::{Block, Widget}};
use tokio::sync::mpsc;

#[derive(Default)]
pub struct PingComponent {
    tcp_server: TcpTest,
    ping: TcpTestLatency,
    latency_rx: Option<mpsc::UnboundedReceiver<TcpTestLatency>>,
    measuring_ping: bool,
}

impl PingComponent {
    pub fn new(tcp_server: TcpTest) -> Self {
        let (_, rx) = mpsc::unbounded_channel();
        Self {
            tcp_server,
            ping: TcpTestLatency::default(),
            latency_rx: Some(rx),
            measuring_ping: false,
        }
    }

    pub fn set_server(&mut self, server: Server) {
        self.tcp_server = TcpTest::new(server.host, 8080);
        self.ping = TcpTestLatency::default();
        self.measuring_ping = false;
        self.latency_rx = None; // Reset the receiver
    }
        
    pub fn start_latency_measurement(&mut self) {
        self.measuring_ping = true;
        let tcp_server = self.tcp_server.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.latency_rx = Some(rx);
        tokio::spawn(async move {
            let response = tcp_server.measure_latency_multiple(20).await;
            match response {
                Ok(latency) => {
                    let _ = tx.send(latency);
                }
                Err(e) => {
                    // Enviar un resultado por defecto para desbloquear
                    let _ = tx.send(TcpTestLatency::default());
                }
            }
        });
    }

    pub async fn check_latency_results(&mut self) {
        if let Some(ref mut rx) = self.latency_rx {
            if let Ok(latency) = rx.try_recv() {
                self.ping = latency;
                self.measuring_ping = false;
            }
        }
    }
}

impl Widget for &PingComponent {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
         let ping_res = Text::from(vec![
            Line::from(format!("Average Latency: {:.2} ms", self.ping.avg))
                .bold()
                .blue()
                .centered(),
            Line::from(format!("Min Latency: {:.2} ms", self.ping.min))
                .bold()
                .green()
                .centered(),
            Line::from(format!("Max Latency: {:.2} ms", self.ping.max))
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
