use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub source: String,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    pub installed_at: String,
    pub agents: Vec<String>,
    pub equip_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_date: Option<String>,
}

impl SkillMetadata {
    pub fn write(&self, skill_dir: &Path) -> Result<(), String> {
        let path = skill_dir.join(".equip.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize metadata: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
    }

    pub fn read(skill_dir: &Path) -> Result<Self, String> {
        let path = skill_dir.join(".equip.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {e}", path.display()))
    }
}

pub fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Manual ISO 8601 formatting without chrono
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Compute date from days since 1970-01-01
    let (year, month, day) = days_to_date(days_since_epoch);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Civil calendar algorithm from Howard Hinnant
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Convert a SystemTime to "YYYY-MM-DD" date string.
pub fn system_time_to_date(time: std::time::SystemTime) -> Option<String> {
    let secs = time.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let days = secs / 86400;
    let (year, month, day) = days_to_date(days);
    Some(format!("{:04}-{:02}-{:02}", year, month, day))
}

/// Extract "YYYY-MM-DD" from an ISO 8601 datetime string.
pub fn iso8601_to_date(iso: &str) -> Option<String> {
    if iso.len() >= 10 && iso.as_bytes()[4] == b'-' && iso.as_bytes()[7] == b'-' {
        Some(iso[..10].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_format() {
        let ts = now_iso8601();
        assert!(ts.len() == 20);
        assert!(ts.ends_with('Z'));
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
    }

    #[test]
    fn iso8601_to_date_strips_time() {
        assert_eq!(
            iso8601_to_date("2026-03-18T14:30:00+10:00"),
            Some("2026-03-18".to_string())
        );
        assert_eq!(
            iso8601_to_date("2026-03-18T14:30:00Z"),
            Some("2026-03-18".to_string())
        );
    }

    #[test]
    fn iso8601_to_date_rejects_short() {
        assert_eq!(iso8601_to_date("2026"), None);
    }

    #[test]
    fn system_time_to_date_works() {
        use std::time::{Duration, UNIX_EPOCH};
        let t = UNIX_EPOCH + Duration::from_secs(1774070400);
        let date = system_time_to_date(t).unwrap();
        assert_eq!(date.len(), 10);
        assert!(date.starts_with("2026-03"));
    }
}
