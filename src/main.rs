mod gui;
mod logger;

use crate::gui::WireGui;
use iced::{Application, Settings, Size};

#[tokio::main]
async fn main() {
    logger::set_logger(true);

    let (w, h) = match elevated_command::Command::is_elevated() {
        true => (500.0, 700.0),
        false => (350.0, 100.0),
    };

    let mut settings = Settings::default();
    settings.window.size = Size::new(w, h);
    settings.window.resizable = false;

    WireGui::run(settings).expect("Fail to launch application!")
}
