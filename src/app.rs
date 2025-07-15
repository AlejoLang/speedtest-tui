use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{layout::Layout, widgets::{Block, Paragraph}, DefaultTerminal, Frame};
use tokio::sync::mpsc;
use crate::{download_component::DownloadComponent, http_tester::HttpTester, ping_component::PingComponent, servers::Servers, services::{HttpTestService, HttpTestState}, upload_component::UploadComponent};

pub struct App {
    running: bool,
    servers: Servers,
    test_service: HttpTestService,
    ping_component: PingComponent,
    download_component: DownloadComponent,
    upload_component: UploadComponent,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            servers: Servers::default(),
            test_service: HttpTestService::new(HttpTester::default()),
            ping_component: PingComponent::default(),
            download_component: DownloadComponent::default(),
            upload_component: UploadComponent::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        let server_update_result = self.servers.update_servers().await;
        match server_update_result {
            Ok(_) => {
            }
            Err(e) => {
                eprintln!("Failed to update servers: {}", e);
                return Err(e.into());
            }
        }
        let current_server = self.servers.get_servers()[0].clone(); 
        let url = format!("http://{}", current_server.host);
        self.test_service.set_tester(HttpTester::new(url.as_str()));

        while self.running {
            self.test_service.check_measurments().await;

            if self.test_service.get_testing() {
                if self.test_service.get_state().clone() == HttpTestState::MeasuringDownload {
                    let new_ping_measurment = self.test_service.get_ping_results().clone();
                    self.ping_component.set_ping_measurement(new_ping_measurment);
                }
                if  self.test_service.get_state().clone() == HttpTestState::MeasuringUpload {
                    let new_download_measurment = self.test_service.get_download_results().clone();
                    self.download_component.set_download_measurement(new_download_measurment);
                }
                if self.test_service.get_state().clone() == HttpTestState::Finished {
                    let new_upload_measurment = self.test_service.get_upload_results().clone();
                    self.upload_component.set_upload_measurement(new_upload_measurment);
                }
            }

            terminal.draw(|frame| self.render(frame))?;
            
            let _ = self.handle_crossterm_events();
            
            tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints([
                ratatui::layout::Constraint::Length(6),
                ratatui::layout::Constraint::Min(3),
                ratatui::layout::Constraint::Min(3),
            ].as_ref())
            .split(frame.area());
        frame.render_widget(&self.ping_component, chunks[0]);
        frame.render_widget(&self.download_component, chunks[1]);
        frame.render_widget(&self.upload_component, chunks[2]);
        let url = self.servers.get_servers()[0].host.clone();
        let p = Block::default().title(url.as_str()).borders(ratatui::widgets::Borders::ALL);
        frame.render_widget(p, frame.area());
    }

    fn handle_crossterm_events(&mut self) -> Result<()> {
        // Usar poll para verificar si hay eventos disponibles sin bloquear
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            
            // Medir latencia (no bloqueante)
            (_, KeyCode::Enter) => {
                self.test_service.run_full_test();
            }
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}