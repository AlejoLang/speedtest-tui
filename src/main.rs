mod app;
mod servers;
mod ping_component;
mod download_component;
mod upload_component;
mod http_tester;
mod services;
use app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    Ok(())
}