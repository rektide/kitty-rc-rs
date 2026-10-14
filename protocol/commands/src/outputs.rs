//! Typed output models for kitty remote-control responses.
//!
//! Salvaged from the legacy `WindowInfo` family with its tolerant-unknown
//! decoding preserved: additive upstream fields (window `neighbors` from
//! 0.45, `title_overridden` from 0.46.2, `session_name`/`last_focused_at`/
//! `needs_attention`/`has_activity_since_last_focus` from 0.47, HEAD-only
//! `program_status`, OS window `active_tab_history`) are ignored rather than
//! rejected. `last_focused_at` values come from a monotonic clock and must
//! never be compared across kitty processes.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ProcessInfo {
    pub pid: Option<u64>,
    #[serde(default)]
    pub cmdline: Vec<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowInfo {
    pub id: Option<u64>,
    pub title: Option<String>,
    pub pid: Option<u64>,
    pub cwd: Option<String>,
    #[serde(default)]
    pub cmdline: Vec<String>,
    #[serde(default)]
    pub foreground_processes: Vec<ProcessInfo>,
    pub at_prompt: Option<bool>,
    pub columns: Option<u64>,
    pub created_at: Option<f64>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub in_alternate_screen: Option<bool>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub is_self: Option<bool>,
    pub last_cmd_exit_status: Option<i32>,
    pub last_reported_cmdline: Option<String>,
    pub lines: Option<u64>,
    #[serde(default)]
    pub user_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutOpts {
    /// Layout-specific keys and value types; e.g. tall's bias is fractional,
    /// grid's bias is a list, and splits has a different option vocabulary.
    #[serde(flatten)]
    pub values: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowGroup {
    pub id: u64,
    #[serde(default)]
    pub window_ids: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AllWindows {
    #[serde(default)]
    pub active_group_history: Vec<u64>,
    pub active_group_idx: Option<u64>,
    #[serde(default)]
    pub window_groups: Vec<WindowGroup>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LayoutState {
    pub all_windows: Option<AllWindows>,
    #[serde(default)]
    pub biased_map: HashMap<String, Value>,
    pub class: Option<String>,
    #[serde(default)]
    pub main_bias: Vec<f32>,
    pub opts: Option<LayoutOpts>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TabGroup {
    pub id: u64,
    #[serde(default)]
    pub windows: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TabInfo {
    #[serde(default)]
    pub windows: Vec<WindowInfo>,
    #[serde(default)]
    pub active_window_history: Vec<u64>,
    #[serde(default)]
    pub enabled_layouts: Vec<String>,
    #[serde(default)]
    pub groups: Vec<TabGroup>,
    pub id: Option<u64>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub layout: Option<String>,
    pub layout_opts: Option<LayoutOpts>,
    pub layout_state: Option<LayoutState>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsWindow {
    #[serde(default)]
    pub tabs: Vec<TabInfo>,
    pub background_opacity: Option<f32>,
    pub id: Option<u64>,
    pub is_active: Option<bool>,
    pub is_focused: Option<bool>,
    pub last_focused: Option<bool>,
    pub platform_window_id: Option<u64>,
    pub wm_class: Option<String>,
    pub wm_name: Option<String>,
}

/// Parse the `data` field of an `ls` response: upstream encodes the listing
/// as a JSON **string**; some transports/tests may hand over the decoded
/// value directly. Both are accepted.
pub fn parse_ls_data(data: &Value) -> Result<Vec<OsWindow>, serde_json::Error> {
    let listing = if let Some(encoded) = data.as_str() {
        serde_json::from_str::<Value>(encoded)?
    } else {
        data.clone()
    };
    serde_json::from_value(listing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_listing_from_encoded_string() {
        let data = json!(r#"[{"id": 1, "tabs": []}]"#.to_string());
        let instances = parse_ls_data(&data).unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, Some(1));
    }

    #[test]
    fn parses_already_decoded_listing() {
        let data = json!([{"id": 7, "tabs": [{"id": 2, "windows": []}]}]);
        let instances = parse_ls_data(&data).unwrap();
        assert_eq!(instances[0].id, Some(7));
        assert_eq!(instances[0].tabs[0].id, Some(2));
    }

    /// A non-list payload is a decode error, not a fabricated empty listing.
    #[test]
    fn rejects_non_list_data() {
        assert!(parse_ls_data(&json!("not json")).is_err());
        assert!(parse_ls_data(&json!({"id": 1})).is_err());
    }

    #[test]
    fn tolerates_unknown_additive_fields() {
        let data = json!(
            r#"[{"id": 1, "tabs": [], "active_tab_history": [1, 2], "future_field": true}]"#
                .to_string()
        );
        assert!(parse_ls_data(&data).is_ok());
    }
}
