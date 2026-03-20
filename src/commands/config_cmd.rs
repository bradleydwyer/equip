use crate::config;
use crate::output;
use crate::telemetry;

pub fn run(key: Option<&str>, value: Option<&str>) -> Result<(), String> {
    let mut settings = config::read_settings()?;

    match (key, value) {
        // No args: show all settings
        (None, _) => {
            let telemetry_status = if telemetry::is_enabled() { "on" } else { "off" };
            println!("equip settings:\n");
            println!(
                "  {:<20} {}",
                output::bold("projects_path"),
                settings.projects_path.as_deref().unwrap_or("(not set)")
            );
            println!("  {:<20} {}", output::bold("telemetry"), telemetry_status);
            println!("\nSet with: {}", output::dim("equip config <key> <value>"));
        }

        // Key only: show that setting
        (Some(k), None) => match k {
            "projects_path" => {
                println!(
                    "{}",
                    settings.projects_path.as_deref().unwrap_or("(not set)")
                );
            }
            "telemetry" => {
                println!("{}", if telemetry::is_enabled() { "on" } else { "off" });
            }
            _ => return Err(format!("Unknown setting: '{k}'")),
        },

        // Key + value: set it
        (Some(k), Some(v)) => match k {
            "projects_path" => {
                let path = if v == "unset" || v.is_empty() {
                    settings.projects_path = None;
                    config::write_settings(&settings)?;
                    println!("{} Cleared projects_path", output::green("✓"));
                    return Ok(());
                } else {
                    v.to_string()
                };
                settings.projects_path = Some(path.clone());
                config::write_settings(&settings)?;
                println!("{} Set projects_path = {}", output::green("✓"), path);
            }
            "telemetry" => {
                let enabled = match v {
                    "on" | "true" | "1" => true,
                    "off" | "false" | "0" => false,
                    _ => return Err("telemetry must be 'on' or 'off'".to_string()),
                };
                telemetry::set_enabled(enabled)?;
                println!(
                    "{} Telemetry {}",
                    output::green("✓"),
                    if enabled { "enabled" } else { "disabled" }
                );
            }
            _ => return Err(format!("Unknown setting: '{k}'")),
        },
    }

    Ok(())
}
