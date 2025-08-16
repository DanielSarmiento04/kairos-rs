use std::io::Write;
use env_logger::Builder;
use log::LevelFilter;
use chrono::Local;
use std::env;

pub fn configure_logger() {
    // If NO_COLOR is set in the environment, disable coloring and bolding.
    let no_color = env::var("NO_COLOR").is_ok();

    Builder::new()
        .format(move |buf, record| {
            let level = record.level();
            let level_str = level.to_string();

            // When colors are enabled we start bold for the whole line and color the level.
            let (prefix, colored_level, suffix) = if no_color {
                ("".to_string(), level_str, "".to_string())
            } else {
                // Bold prefix for entire line
                let prefix = "\x1b[1m".to_string();
                // Use color for the level but reset only the foreground (39) so bold remains
                let colored = match level {
                    log::Level::Error   => format!("\x1b[31m{}\x1b[39m", level_str), // red
                    log::Level::Warn    => format!("\x1b[33m{}\x1b[39m", level_str), // yellow
                    log::Level::Info    => format!("\x1b[32m{}\x1b[39m", level_str), // green
                    log::Level::Debug   => format!("\x1b[34m{}\x1b[39m", level_str), // blue
                    log::Level::Trace   => format!("\x1b[35m{}\x1b[39m", level_str), // magenta
                };
                let suffix = "\x1b[0m".to_string(); // final reset of bold/color at line end
                (prefix, colored, suffix)
            };

            writeln!(
                buf,
                "{}{} [{}] | {}:{} | {}{}",
                prefix,
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                colored_level,
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args(),
                suffix,
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}
