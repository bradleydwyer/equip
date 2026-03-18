use crate::config;
use crate::output;

pub fn run(key: Option<&str>, value: Option<&str>) -> Result<(), String> {
    let mut settings = config::read_settings()?;

    match (key, value) {
        // No args: show all settings
        (None, _) => {
            println!("equip settings:\n");
            println!(
                "  {:<20} {}",
                output::bold("projects_path"),
                settings.projects_path.as_deref().unwrap_or("(not set)")
            );
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
            _ => return Err(format!("Unknown setting: '{k}'")),
        },
    }

    Ok(())
}
