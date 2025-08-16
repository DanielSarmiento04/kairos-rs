# Ben

Logger and run instructions

This project uses a custom logger configured in `src/logs/logger.rs`.

Quick start

- Run with the default log level (Info, Warn, Error):

  ```zsh
  cargo run
  ```

Enable debug/trace output

The logger enforces Info level by default in code. To see Debug and Trace messages you have two options:

1) Temporary (no code change):

   - If `src/logs/logger.rs` does NOT call `log::set_max_level(...)`, you can enable verbose logging at runtime with:

     ```zsh
     RUST_LOG=trace cargo run
     # or just debug level:
     RUST_LOG=debug cargo run
     # to enable trace only for this crate:
     RUST_LOG=ben=trace cargo run
     ```

2) Make a small code change (permanent):

   - Open `src/logs/logger.rs` and remove or comment out the line:

     ```rust
     log::set_max_level(LevelFilter::Info);
     ```

   - Then run with `RUST_LOG` as shown above.

Colors

- The logger emits ANSI colors and bold by default when the output is a TTY.
- To force-disable colors/bolding, set `NO_COLOR` in your environment when running:

  ```zsh
  NO_COLOR=1 cargo run
  ```

Timestamp format

- Timestamps are formatted like `Aug 16 04:06:39 PM`.

Notes

- Use the `RUST_LOG` environment variable to control per-crate/module filtering. Example: `RUST_LOG=ben::submodule=debug cargo run`.
- If you want automatic behavior (colors only on TTY, or more advanced formatting), I can update the logger to use a small crate, but this repo currently implements colors without extra dependencies.

