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
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::env;

const RSS_SCHEDULER: &str = "rss_scheduler";
const EMBEDDING_SCHEDULER: &str = "embedding_scheduler";
const FOLDER_SCHEDULER: &str = "folder_scheduler";
const NEWS_SCHEDULER: &str = "news_scheduler";
const OMNINEWS_SUBSCRIPTION_SCHEDULER: &str = "omninews_subscription_scheduler";
const SUBSCRIPTION_SCHEDULER: &str = "subscription_scheduler";
const USER_SCHEDULER: &str = "user_scheduler";

pub fn load_logger() {
    // Disable ANSI colors in log4rs output
    env::set_var("RUST_LOG_STYLE", "never");
    // Disable ANSI colors in Rocket CLI output
    env::set_var("ROCKET_CLI_COLORS", "true");
    // Set default log level to "info" if RUST_LOG is not already set
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Console log
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{d}] {h({l})} - {m} {n}")))
        .build();

    let create_file_appender = |name: &str| -> RollingFileAppender {
        let log_path = format!("logs/{name}.log");
        let size_trigger = SizeTrigger::new(10 * 1024 * 1024); // 10MB
        let rollder = FixedWindowRoller::builder()
            .build(&format!("logs/{name}.log.{{}}"), 5)
            .unwrap();
        let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(rollder));

        RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} {h({l})} - {m}{n}")))
            .build(log_path, Box::new(policy))
            .unwrap_or_else(|_| panic!("Failed to create file appender for {name}"))
    };

    let rss_appender = create_file_appender(RSS_SCHEDULER);
    let embedding_appender = create_file_appender(EMBEDDING_SCHEDULER);
    let folder_appender = create_file_appender(FOLDER_SCHEDULER);
    let news_appender = create_file_appender(NEWS_SCHEDULER);
    let omninews_subscription_appender = create_file_appender(OMNINEWS_SUBSCRIPTION_SCHEDULER);
    let subscription_appender = create_file_appender(SUBSCRIPTION_SCHEDULER);
    let user_appender = create_file_appender(USER_SCHEDULER);

    let server_appender = create_file_appender("server");

    let mut config_builder = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("server_file", Box::new(server_appender)))
        .appender(Appender::builder().build("rss_file", Box::new(rss_appender)))
        .appender(Appender::builder().build("embedding_file", Box::new(embedding_appender)))
        .appender(Appender::builder().build("folder_file", Box::new(folder_appender)))
        .appender(Appender::builder().build("news_file", Box::new(news_appender)))
        .appender(Appender::builder().build(
            "omninews_subscription_file",
            Box::new(omninews_subscription_appender),
        ))
        .appender(Appender::builder().build("subscription_file", Box::new(subscription_appender)))
        .appender(Appender::builder().build("user_file", Box::new(user_appender)));

    config_builder = config_builder
        .logger(
            Logger::builder()
                .appender("rss_file")
                .build(RSS_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("embedding_file")
                .build(EMBEDDING_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("folder_file")
                .build(FOLDER_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("news_file")
                .build(NEWS_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("omninews_subscription_file")
                .build(OMNINEWS_SUBSCRIPTION_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("subscription_file")
                .build(SUBSCRIPTION_SCHEDULER, LevelFilter::Info),
        )
        .logger(
            Logger::builder()
                .appender("user_file")
                .build(USER_SCHEDULER, LevelFilter::Info),
        );

    let config = config_builder
        .build(
            Root::builder()
                .appender("stdout")
                .appender("server_file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();
}

#[macro_export]
macro_rules! server_info {
    ($($arg:tt)+) => {
        log::info!(target: "server", $($arg)+)
    };
}

#[macro_export]
macro_rules! server_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "server", $($arg)+)
    };
}

#[macro_export]
macro_rules! server_error {
    ($($arg:tt)+) => {
        log::error!(target: "server", $($arg)+)
    };
}

#[macro_export]
macro_rules! rss_info {
    ($($arg:tt)+) => {
        log::info!(target: "rss_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! rss_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "rss_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! rss_error {
    ($($arg:tt)+) => {
        log::error!(target: "rss_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! embedding_info {
    ($($arg:tt)+) => {
        log::info!(target: "embedding_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! embedding_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "embedding_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! embedding_error {
    ($($arg:tt)+) => {
        log::error!(target: "embedding_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! folder_info {
    ($($arg:tt)+) => {
        log::info!(target: "folder_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! folder_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "folder_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! folder_error {
    ($($arg:tt)+) => {
        log::error!(target: "folder_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! news_info {
    ($($arg:tt)+) => {
        log::info!(target: "news_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! news_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "news_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! news_error {
    ($($arg:tt)+) => {
        log::error!(target: "news_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! omninews_subscription_info {
    ($($arg:tt)+) => {
        log::info!(target: "omninews_subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! omninews_subscription_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "omninews_subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! omninews_subscription_error {
    ($($arg:tt)+) => {
        log::error!(target: "omninews_subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! subscription_info {
    ($($arg:tt)+) => {
        log::info!(target: "subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! subscription_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! subscription_error {
    ($($arg:tt)+) => {
        log::error!(target: "subscription_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! user_info {
    ($($arg:tt)+) => {
        log::info!(target: "user_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! user_warn {
    ($($arg:tt)+) => {
        log::warn!(target: "user_scheduler", $($arg)+)
    };
}

#[macro_export]
macro_rules! user_error {
    ($($arg:tt)+) => {
        log::error!(target: "user_scheduler", $($arg)+)
    };
}
