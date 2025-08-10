use anyhow::Result;
use std::env;

#[cfg(feature = "cli")]
use abcdeez_core::ui::tui;

use abcdeez_core::demo;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "demo" => {
                demo::run_demo();
                demo::demonstrate_task_types();
                demo::demonstrate_dag_tasks();
                demo::demonstrate_statistical_analysis();
                demo::demonstrate_eig();
                demo::demonstrate_extended_tasks();
                Ok(())
            }
            #[cfg(feature = "cli")]
            "legacy" => {
                // Use old UI for compatibility
                abcdeez_core::ui::tui::run().map_err(Into::into)
            }
            _ => {
                #[cfg(feature = "cli")]
                {
                    tui::run()
                }
                #[cfg(not(feature = "cli"))]
                {
                    println!("CLI feature not enabled. Run with --features cli");
                    Ok(())
                }
            }
        }
    } else {
        #[cfg(feature = "cli")]
        {
            tui::run()
        }
        #[cfg(not(feature = "cli"))]
        {
            println!("CLI feature not enabled. Run with --features cli");
            Ok(())
        }
    }
}
