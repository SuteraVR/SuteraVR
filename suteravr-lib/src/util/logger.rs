pub trait Logger {
    fn write_info(&self, data: String);
    fn write_warn(&self, data: String);
    fn write_error(&self, data:String);
    fn write_debug(&self, data:String);
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)+) => {
        use $crate::util::logger::Logger;
        $logger.write_info(std::format!($($arg)+))
    }
}
