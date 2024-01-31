use log;
pub trait Logger {
    fn write_info(&self, data: String);
    fn write_warn(&self, data: String);
    fn write_error(&self, data: String);
    fn write_debug(&self, data: String);
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)+) => {    
        (&$logger as &dyn $crate::util::logger::Logger).write_info(std::format!($($arg)+))
    }
}
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)+) => {
        (&$logger as &dyn $crate::util::logger::Logger).write_warn(std::format!($($arg)+))
    }
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)+) => {
        (&$logger as &dyn $crate::util::logger::Logger).write_error(std::format!($($arg)+))
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)+) => {
        (&$logger as &dyn $crate::util::logger::Logger).write_debug(std::format!($($arg)+))
    }
}

#[derive(Clone)]
pub struct EnvLogger {
    pub target: String,
}

impl Logger for EnvLogger {
    fn write_info(&self, data: String) {
        log::info!(target: self.target.as_str(), "{}", data);
    }

    fn write_warn(&self, data: String) {
        log::warn!(target: self.target.as_str(), "{}", data);
    }

    fn write_error(&self, data: String) {
        log::error!(target: self.target.as_str(), "{}", data);
    }

    fn write_debug(&self, data: String) {
        log::debug!(target: self.target.as_str(), "{}", data);
    }
}
