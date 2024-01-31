use godot::log::godot_print;
use suteravr_lib::util::logger::Logger;
#[derive(Clone)]
pub struct GodotLogger {
    pub target: String
}

impl Logger for GodotLogger {
    fn write_debug(&self, data: String) {
        godot_print!("[DEBUG {}] {}", self.target, data);
    }
    fn write_info(&self, data: String) {
        godot_print!("[INFO  {}] {}", self.target, data);
    }
    fn write_warn(&self, data: String) {
        godot_print!("[WARN  {}] {}", self.target, data);
    }
    fn write_error(&self, data: String) {
        godot_print!("[ERROR {}] {}", self.target, data);
    }
}