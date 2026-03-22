use dialoguer::{FuzzySelect, theme::ColorfulTheme};

use crate::commands;
use crate::output;

const REGISTRY_URL: &str = "https://registry.equip.codes";
const API_VERSION: &str = "v1";

#[derive(serde::Deserialize)]
struct SearchResponse {
    skills: Vec<SkillResult>,
    total: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SkillResult {
    name: String,
    description: Option<String>,
    install_cmd: String,
    sources: Vec<String>,
    installs: u64,
}

pub fn run(
    query: &[String],
    source: Option<&str>,
    sort: &str,
    limit: usize,
    json: bool,
) -> Result<(), String> {
    let q = query.join(" ");

    let mut url = format!(
        "{REGISTRY_URL}/{API_VERSION}/search?q={}&limit={limit}&sort={}",
        encode(&q),
        encode(sort),
    );
    if let Some(s) = source {
        url.push_str(&format!("&source={}", encode(s)));
    }

    let spinner = if !json {
        Some(output::Spinner::start("Searching"))
    } else {
        None
    };

    let response: SearchResponse = ureq::get(&url)
        .call()
        .map_err(|e| format!("Search request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    if let Some(s) = spinner {
        s.stop();
    }

    if json {
        print_json(&response)?;
    } else {
        interactive_select(&q, &response)?;
    }

    Ok(())
}

fn interactive_select(query: &str, response: &SearchResponse) -> Result<(), String> {
    if response.skills.is_empty() {
        println!("No results for {}", output::bold(&format!("\"{query}\"")));
        return Ok(());
    }

    println!(
        "Search results for {} ({} found)\n",
        output::bold(&format!("\"{query}\"")),
        format_number(response.total),
    );

    let max_name = response
        .skills
        .iter()
        .map(|s| s.name.len())
        .max()
        .unwrap_or(0);
    let name_width = max_name + 2;

    let items: Vec<String> = response
        .skills
        .iter()
        .map(|s| {
            let padded_name = format!("{:<width$}", s.name, width = name_width);
            let desc = s
                .description
                .as_deref()
                .map(|d| truncate(d, 52))
                .unwrap_or_default();
            format!(
                "{}  {}  {}",
                padded_name,
                desc,
                format_installs(s.installs),
            )
        })
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .highlight_matches(true)
        .interact_opt()
        .map_err(|e| format!("Selection failed: {e}"))?;

    let Some(idx) = selection else {
        return Ok(());
    };

    let skill = &response.skills[idx];
    println!(
        "\nInstalling {} ...\n",
        output::bold(&skill.install_cmd),
    );
    commands::install::run(&skill.install_cmd, true, &[], false, false)
}

fn print_json(response: &SearchResponse) -> Result<(), String> {
    let entries: Vec<serde_json::Value> = response
        .skills
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "description": s.description,
                "install_cmd": s.install_cmd,
                "sources": s.sources,
                "installs": s.installs,
            })
        })
        .collect();

    let out = serde_json::json!({
        "skills": entries,
        "total": response.total,
    });

    let json =
        serde_json::to_string_pretty(&out).map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    println!("{json}");
    Ok(())
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_installs(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M installs", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K installs", n as f64 / 1_000.0)
    } else {
        format!("{n} installs")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Find a safe char boundary at or before max
    let mut end = max.min(s.len());
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let safe = &s[..end];
    // Find first sentence boundary
    if let Some(pos) = safe.find(". ") {
        return format!("{}.", &s[..pos]);
    }
    let mut trunc_end = (max - 3).min(s.len());
    while trunc_end > 0 && !s.is_char_boundary(trunc_end) {
        trunc_end -= 1;
    }
    format!("{}...", &s[..trunc_end])
}

fn encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            ' ' => result.push('+'),
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    result
}
