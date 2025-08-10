//! Demo application for the research dashboard with pre-registration interface

use abcdeez_core::ui::dashboard::run_research_dashboard;
use std::io;

fn main() -> io::Result<()> {
    println!("Starting Research Dashboard Demo...");
    println!("This demonstrates the pre-registration interface for scientific studies.");
    println!();

    // Run the research dashboard
    run_research_dashboard()?;

    println!();
    println!("Research Dashboard Demo completed.");

    Ok(())
}
