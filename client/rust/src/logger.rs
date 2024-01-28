#[macro_export]
macro_rules! log {
    ($fmt:literal $(, $args:expr)* $(,)?) => {
        godot::log::godot_print!(
            "[{}] {}",
            LOGGER_CONTEXT,
            format!($fmt $(, $args)*)
        )
    };
}

#[macro_export]
macro_rules! set_logger_target {
    ($target:expr) => {
        pub const LOGGER_CONTEXT: &str = $target;
    };
}
