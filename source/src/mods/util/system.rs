use crate::mods::console::output::{eprintln, println};
pub fn exit_with_error(message: String) {
    eprintln(format!("[ERROR]Error: {}", message));
    std::process::exit(1)
}
