#![allow(dead_code)]

use log::LevelFilter;
use env_logger::TimestampPrecision;

use crate::logger::LogTarget;

// Build Constants:
pub(crate) const APP_NAME:   &str = "Stray Cat";
pub(crate) const BUILD_NAME: &str = env!("BUILD_NAME");
pub(crate) const VERSION_ID: &str = env!("CARGO_PKG_VERSION");
pub(crate) const BUILD_TIME: &str = env!("BUILD_TIME");
pub(crate) const BUILD_ID:   &str = env!("BUILD_ID");
pub(crate) const AUTHORS:    &str = env!("CARGO_PKG_AUTHORS");
pub(crate) const COPYRIGHT:  &str = "2026";
pub(crate) const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub(crate) const LICENSE:    &str = env!("CARGO_PKG_LICENSE");

// Logging Constants:
pub(crate) const DEFAULT_LOG_LEVEL:            LevelFilter                                 = LevelFilter::Info;
pub(crate) const DEFAULT_LOG_FORMAT_TIMESTAMP: Option<env_logger::fmt::TimestampPrecision> = Some(TimestampPrecision::Millis); // None;
pub(crate) const DEFAULT_LOG_FORMAT_LEVEL:     bool                                        = true; // false;
pub(crate) const DEFAULT_LOG_FORMAT_TARGET:    bool                                        = true; // false;
pub(crate) const DEFAULT_LOG_DIR:              &'static str                                = "logs";
pub(crate) const DEFAULT_LOG_FILE:             &'static str                                = "straycat.log";
pub(crate) const DEFAULT_LOG_TARGET:           LogTarget                                   = LogTarget::Stdout;
//pub(crate) const DEFAULT_LOG_TARGET:           LogTarget                                   = LogTarget::File(DEFAULT_LOG_DIR, DEFAULT_LOG_FILE);
pub(crate) const DEFAULT_LOG_VERBOSE:          bool                                        = false;
pub(crate) const DEFAULT_LOG_DEBUG:            bool                                        = false;

pub(crate) const DEFAULT_LOG_NO_DEBUG_MODULES: &[&str] = &[
  "ureq",
  "multipart_ng::server::boundary",
];

// Stray Cat Defaults:
pub(crate) const DEFAULT_MODE: &str = "batch";
pub(crate) const MODE_LIST:    &[&str] = &[ "batch","interactive","help"];
pub(crate) const DEFAULT_WAIT: u64  = 3;

// Splash:
pub(crate) const SPLASH_COLOUR: u8    = 95;
pub(crate) const SPLASH_TXT:    &str  = include_str!("../resources/straycat.txt");

// Includes:
pub(crate) const HELP_TEMPLATE: &str  = include_str!("../resources/help.tpl");

