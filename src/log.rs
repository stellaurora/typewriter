//! Logging integration
use env_logger::{
    Env,
    fmt::style::{AnsiColor, Color, Style},
};
use std::io::Write;

pub fn setup_logging() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            let level_color = Some(Color::from(match record.level() {
                log::Level::Error => AnsiColor::Red,
                log::Level::Warn => AnsiColor::Yellow,
                log::Level::Info => AnsiColor::Green,
                log::Level::Debug => AnsiColor::Blue,
                log::Level::Trace => AnsiColor::Cyan,
            }));

            // Level for the level text which should pop out
            let level_style = Style::new().fg_color(level_color).bold();
            let msg_style = Style::new().fg_color(level_color);

            writeln!(
                buf,
                "[{level_style}{}{level_style:#}] {msg_style}{}{msg_style:#}",
                record.level(),
                record.args()
            )
        })
        .init();
}
