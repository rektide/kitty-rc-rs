//! Tab/layout finite family. Layout names permit options and future layouts.
use crate::targets::{Layout, Rgb, TabTarget};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TabColor {
    ActiveFg,
    ActiveBg,
    InactiveFg,
    InactiveBg,
}

#[derive(Debug, Clone)]
pub struct TabCommand {
    pub target: TabTarget,
    pub operation: TabOperation,
}
#[derive(Debug, Clone)]
pub enum TabOperation {
    Focus,
    Close {
        ignore_no_match: bool,
    },
    Title(String),
    /// None creates a new OS window; otherwise uses first matched tab's OS window.
    Detach {
        target_tab: Option<String>,
    },
    GotoLayout(Layout),
    LastUsedLayout,
    EnabledLayouts {
        layouts: Vec<Layout>,
        configured: bool,
    },
    Colors(BTreeMap<TabColor, Option<Rgb>>),
}
impl Command for TabCommand {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        let name = match &self.operation {
            TabOperation::Focus => "focus-tab",
            TabOperation::Close { ignore_no_match } => {
                p["ignore_no_match"] = json!(ignore_no_match);
                "close-tab"
            }
            TabOperation::Title(title) => {
                p["title"] = json!(title);
                "set-tab-title"
            }
            TabOperation::Detach { target_tab } => {
                p["target_tab"] = json!(target_tab);
                "detach-tab"
            }
            TabOperation::GotoLayout(layout) => {
                p["layout"] = json!(layout.as_str());
                "goto-layout"
            }
            TabOperation::LastUsedLayout => {
                p["all"] = json!(matches!(self.target, TabTarget::All));
                "last-used-layout"
            }
            TabOperation::EnabledLayouts {
                layouts,
                configured,
            } => {
                p["layouts"] = json!(layouts.iter().map(Layout::as_str).collect::<Vec<_>>());
                p["configured"] = json!(configured);
                "set-enabled-layouts"
            }
            TabOperation::Colors(colors) => {
                p["colors"] = json!(colors);
                "set-tab-color"
            }
        };
        request(name, p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl CanSubmitWithoutReply for TabCommand {}
