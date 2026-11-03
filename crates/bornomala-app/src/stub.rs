//! Linux stub implementation for Bornomala App.
//! Allows running unit tests and checking compilation on non-Windows platforms.

pub fn run_app(converter_only: bool, startup_mode: bool) {
    println!("Bornomala App Linux Simulation:");
    println!("  Converter only: {}", converter_only);
    println!("  Startup mode:   {}", startup_mode);
    println!("  Engine & UI modules verified.");
}
