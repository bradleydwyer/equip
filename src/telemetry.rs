use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config;

const ENDPOINT: &str = "https://registry.equip.codes/v1/events";

#[derive(Serialize)]
struct Event {
    client_id: String,
    event: String,
    equip_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    skill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    os: String,
    arch: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct TelemetryState {
    client_id: String,
    #[serde(default)]
    enabled: bool,
}

fn state_path() -> Result<PathBuf, String> {
    Ok(config::equip_dir()?.join("telemetry.json"))
}

fn load_or_create_state() -> Result<TelemetryState, String> {
    let path = state_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read telemetry state: {e}"))?;
        return serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse telemetry state: {e}"));
    }

    // First run: generate client ID, default to enabled
    let state = TelemetryState {
        client_id: generate_id(),
        enabled: true,
    };
    save_state(&state)?;
    Ok(state)
}

fn save_state(state: &TelemetryState) -> Result<(), String> {
    let dir = config::equip_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;
    let path = state_path()?;
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize telemetry state: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

fn generate_id() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut h = s.build_hasher();
    h.write_u128(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    );
    let a = h.finish();
    let mut h2 = s.build_hasher();
    h2.write_usize(std::process::id() as usize);
    let b = h2.finish();
    format!("{a:016x}{b:016x}")
}

/// Enable or disable telemetry. Returns the new state for confirmation.
pub fn set_enabled(enabled: bool) -> Result<bool, String> {
    let mut state = load_or_create_state()?;
    state.enabled = enabled;
    save_state(&state)?;
    Ok(state.enabled)
}

/// Check if telemetry is enabled.
pub fn is_enabled() -> bool {
    load_or_create_state().map(|s| s.enabled).unwrap_or(false)
}

/// Fire-and-forget: send an event in a background thread.
/// Never blocks, never fails visibly.
pub fn send(event_name: &str, skill: Option<&str>, source: Option<&str>) {
    let state = match load_or_create_state() {
        Ok(s) if s.enabled => s,
        _ => return,
    };

    let event = Event {
        client_id: state.client_id,
        event: event_name.to_string(),
        equip_version: env!("CARGO_PKG_VERSION").to_string(),
        skill: skill.map(String::from),
        source: source.map(String::from),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    };

    let body = match serde_json::to_string(&event) {
        Ok(b) => b,
        Err(_) => return,
    };

    std::thread::spawn(move || {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .new_agent();
        let _ = agent
            .post(ENDPOINT)
            .content_type("application/json")
            .send(body.as_bytes());
    });
}
