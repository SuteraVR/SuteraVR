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
