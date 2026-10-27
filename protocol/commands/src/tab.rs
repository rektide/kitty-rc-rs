//! Tab/layout finite family. Layout names permit options and future layouts.
use crate::targets::{Layout, Rgb, TabId, TabMatch, TabTarget};
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

/// The destination OS window for a detached tab, identified by a contained tab.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TabDetachDestination {
    #[default]
    NewOsWindow,
    Tab(TabMatch),
}
impl TabDetachDestination {
    pub fn tab(tab: impl Into<TabMatch>) -> Self {
        Self::Tab(tab.into())
    }
    fn wire(&self) -> Option<&str> {
        match self {
            Self::NewOsWindow => None,
            Self::Tab(tab) => Some(tab.as_str()),
        }
    }
}
impl From<TabId> for TabDetachDestination {
    fn from(id: TabId) -> Self {
        Self::tab(id)
    }
}
impl From<TabMatch> for TabDetachDestination {
    fn from(tab: TabMatch) -> Self {
        Self::Tab(tab)
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub struct TabCommand {
    #[builder(default, into)]
    pub target: TabTarget,
    pub operation: TabOperation,
}
impl TabCommand {
    pub fn new(target: impl Into<TabTarget>, operation: TabOperation) -> Self {
        Self {
            target: target.into(),
            operation,
        }
    }
    pub fn focus(target: impl Into<TabTarget>) -> Self {
        Self::new(target, TabOperation::Focus)
    }
    pub fn close(target: impl Into<TabTarget>) -> Self {
        Self::new(
            target,
            TabOperation::Close {
                ignore_no_match: false,
            },
        )
    }
    pub fn title(target: impl Into<TabTarget>, title: impl Into<String>) -> Self {
        Self::new(target, TabOperation::Title(title.into()))
    }
    pub fn detach(
        target: impl Into<TabTarget>,
        destination: impl Into<TabDetachDestination>,
    ) -> Self {
        Self::new(
            target,
            TabOperation::Detach {
                destination: destination.into(),
            },
        )
    }
    pub fn layout(target: impl Into<TabTarget>, layout: Layout) -> Self {
        Self::new(target, TabOperation::GotoLayout(layout))
    }
    pub fn last_used_layout(target: impl Into<TabTarget>) -> Self {
        Self::new(target, TabOperation::LastUsedLayout)
    }
    pub fn enabled_layouts(
        target: impl Into<TabTarget>,
        layouts: impl IntoIterator<Item = Layout>,
    ) -> Self {
        Self::new(
            target,
            TabOperation::EnabledLayouts {
                layouts: layouts.into_iter().collect(),
                configured: false,
            },
        )
    }
}
#[derive(Debug, Clone)]
pub enum TabOperation {
    Focus,
    Close {
        ignore_no_match: bool,
    },
    Title(String),
    Detach {
        destination: TabDetachDestination,
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
            TabOperation::Detach { destination } => {
                p["target_tab"] = json!(destination.wire());
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
