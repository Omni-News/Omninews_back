use dotenv::dotenv;
use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{
                roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
            },
            RollingFileAppender,
        },
    },
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::env;

pub fn load_env() {
    dotenv().ok();
}

pub fn configure_logging() {
    // Disable ANSI colors in log4rs output
    env::set_var("RUST_LOG_STYLE", "never");
    // Disable ANSI colors in Rocket CLI output
    env::set_var("ROCKET_CLI_COLORS", "false");
    // Set default log level to "info" if RUST_LOG is not already set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Console log
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{d}] {h({l})} - {m} {n}")))
        .build();

    let log_path = "server.log";

    // Rolling file policy (10MB per file, keep 5 backups)
    let size_trigger = SizeTrigger::new(10 * 1024 * 1024); // 10MB
    let roller = FixedWindowRoller::builder()
        .build("server.log.{}", 5)
        .unwrap();
    let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(roller));

    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {h({l})} - {m}{n}")))
        .build(log_path, Box::new(policy))
        .expect("Failed to create file appender");

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(
            Appender::builder()
                .filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
                    LevelFilter::Warn,
                )))
                .build("file", Box::new(file_appender)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}
