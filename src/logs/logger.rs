use std::io::Write;
use env_logger::Builder;
use log::LevelFilter;
use chrono::Local;
use std::env;

// Adjust these widths to control alignment
const LEVEL_FIELD_WIDTH: usize = 8; // visible width for the '[LEVEL]' column including padding
const FILE_LINE_FIELD_WIDTH: usize = 22; // visible width for the 'file:line' column including padding

// Compute visible length of a string while stripping simple ANSI escape sequences (\x1b[...m)
fn visible_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut visible = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            // ESC detected, skip until 'm' or end
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
            }
            while i < bytes.len() {
                let b = bytes[i];
                i += 1;
                if b == b'm' {
                    break;
                }
            }
        } else {
            // Determine UTF-8 code point length to advance correctly, but count as one visible character
            let first = bytes[i];
            let width = if first < 0x80 {
                1
            } else if first >> 5 == 0b110 {
                2
            } else if first >> 4 == 0b1110 {
                3
            } else if first >> 3 == 0b11110 {
                4
            } else {
                1
            };
            visible += 1;
            i += width;
        }
    }
    visible
}

pub fn configure_logger() {
    // If NO_COLOR is set in the environment, disable coloring and bolding.
    let no_color = env::var("NO_COLOR").is_ok();

    Builder::new()
        .format(move |buf, record| {
            let level = record.level();
            let level_plain = level.to_string();

            // When colors are enabled we start bold for the whole line and color the level.
            let (prefix, colored_level, suffix) = if no_color {
                ("".to_string(), level_plain.clone(), "".to_string())
            } else {
                // Bold prefix for entire line
                let prefix = "\x1b[1m".to_string();
                // Use color for the level but reset only the foreground (39) so bold remains
                let colored = match level {
                    log::Level::Error => format!("\x1b[31m{}\x1b[39m", level_plain), // red
                    log::Level::Warn  => format!("\x1b[33m{}\x1b[39m", level_plain), // yellow
                    log::Level::Info  => format!("\x1b[32m{}\x1b[39m", level_plain), // green
                    log::Level::Debug => format!("\x1b[34m{}\x1b[39m", level_plain), // blue
                    log::Level::Trace => format!("\x1b[35m{}\x1b[39m", level_plain), // magenta
                };
                let suffix = "\x1b[0m".to_string(); // final reset of bold/color at line end
                (prefix, colored, suffix)
            };

            // Build level display like "[INFO]"
            let level_display = format!("[{}]", colored_level);
            let level_vis_len = visible_len(&level_display);
            let level_padding = if level_vis_len >= LEVEL_FIELD_WIDTH {
                1
            } else {
                LEVEL_FIELD_WIDTH - level_vis_len
            };

            // Build file:line
            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            let file_line = format!("{}:{}", file, line);
            let file_line_vis_len = visible_len(&file_line);
            let file_line_padding = if file_line_vis_len >= FILE_LINE_FIELD_WIDTH {
                1
            } else {
                FILE_LINE_FIELD_WIDTH - file_line_vis_len
            };

            writeln!(
                buf,
                "{}{} | {}{}| {}{}| {}{}",
                prefix,
                Local::now().format("%b %d %y %I:%M:%S %p"),
                level_display,
                " ".repeat(level_padding),
                file_line,
                " ".repeat(file_line_padding),
                record.args(),
                suffix,
            )
        })
        // Only show logs at Info level and above (Info, Warn, Error)
        .filter_level(LevelFilter::Trace)
        .init();

    // Enforce the max log level globally in case other code attempts to lower/raise it
    log::set_max_level(LevelFilter::Trace);
}
