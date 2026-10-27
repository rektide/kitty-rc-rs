//! Finite appearance payloads, not client-side config-file readers or image
//! uploads. Scope fields follow each native command rather than a fake universal
//! window selector.
use crate::targets::{Finite, InvalidValue, Opacity, Rgb, WindowMatch, WindowSet};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde::{Serialize, Serializer};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt;

/// Nonnegative, finite font operand. Zero is only valid for absolute reset;
/// constructing a divide/multiply operation with zero is rejected below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontChange {
    amount: f64,
    operator: Option<char>,
}
impl FontChange {
    pub fn absolute(points: f64) -> Result<Self, InvalidValue> {
        Self::new(points, None)
    }
    pub fn increase(points: f64) -> Result<Self, InvalidValue> {
        Self::new(points, Some('+'))
    }
    pub fn decrease(points: f64) -> Result<Self, InvalidValue> {
        Self::new(points, Some('-'))
    }
    pub fn multiply(factor: f64) -> Result<Self, InvalidValue> {
        Self::new(factor, Some('*'))
    }
    pub fn divide(factor: f64) -> Result<Self, InvalidValue> {
        Self::new(factor, Some('/'))
    }
    fn new(amount: f64, operator: Option<char>) -> Result<Self, InvalidValue> {
        if !amount.is_finite() || amount < 0.0 || (matches!(operator, Some('*' | '/')) && amount == 0.0) {
            return Err(InvalidValue);
        }
        Ok(Self { amount, operator })
    }
    pub fn reset() -> Self {
        Self { amount: 0.0, operator: None }
    }
    pub fn amount(self) -> f64 {
        self.amount
    }
    pub fn operator(self) -> Option<char> {
        self.operator
    }
    pub fn argument(self) -> String {
        format!("{}{}", self.operator.map_or_else(String::new, |c| c.to_string()), self.amount)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LigatureStrategy {
    Never,
    Always,
    Cursor,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgb(Rgb),
    Reset,
    TransparentBackgrounds(String),
}

/// Native color option names. Palette indices cover all 256 terminal colors;
/// custom names explicitly opt into kitty's evolving configuration vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorName {
    Foreground,
    Background,
    Cursor,
    CursorTextColor,
    SelectionForeground,
    SelectionBackground,
    ActiveBorderColor,
    InactiveBorderColor,
    BellBorderColor,
    ActiveTabForeground,
    ActiveTabBackground,
    InactiveTabForeground,
    InactiveTabBackground,
    TabBarBackground,
    TabBarMarginColor,
    UrlColor,
    VisualBellColor,
    TransparentBackgroundColors,
    Palette(u8),
    Custom(String),
}
impl fmt::Display for ColorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Foreground => "foreground",
            Self::Background => "background",
            Self::Cursor => "cursor",
            Self::CursorTextColor => "cursor_text_color",
            Self::SelectionForeground => "selection_foreground",
            Self::SelectionBackground => "selection_background",
            Self::ActiveBorderColor => "active_border_color",
            Self::InactiveBorderColor => "inactive_border_color",
            Self::BellBorderColor => "bell_border_color",
            Self::ActiveTabForeground => "active_tab_foreground",
            Self::ActiveTabBackground => "active_tab_background",
            Self::InactiveTabForeground => "inactive_tab_foreground",
            Self::InactiveTabBackground => "inactive_tab_background",
            Self::TabBarBackground => "tab_bar_background",
            Self::TabBarMarginColor => "tab_bar_margin_color",
            Self::UrlColor => "url_color",
            Self::VisualBellColor => "visual_bell_color",
            Self::TransparentBackgroundColors => "transparent_background_colors",
            Self::Palette(index) => return write!(f, "color{index}"),
            Self::Custom(name) => name,
        })
    }
}
impl Serialize for ColorName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// The nine native logo anchors, or the server's configured position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LogoPosition {
    #[default]
    #[serde(rename = "")]
    Configured,
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpacingEdge {
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackgroundLayout {
    #[default]
    Configured,
    Tiled,
    MirrorTiled,
    Scaled,
    Clamped,
    Centered,
    #[serde(rename = "cscaled")]
    CenteredScaled,
}

/// Unlike uploads these selectors do not enter the shared stream accumulator.
#[derive(Debug, Clone)]
pub enum BackgroundImage {
    Remove {
        configured: bool,
    },
    Index(u32),
    /// A leading sign means increment even when the increment is zero.
    Increment(i32),
}

#[derive(Debug, Clone, bon::Builder)]
pub struct Appearance {
    #[builder(default)]
    pub windows: WindowSet,
    pub operation: AppearanceOperation,
}
impl Appearance {
    pub fn new(windows: WindowSet, operation: AppearanceOperation) -> Self {
        Self { windows, operation }
    }

    pub fn ligatures(windows: WindowSet, strategy: LigatureStrategy) -> Self {
        Self::new(windows, AppearanceOperation::Ligatures(strategy))
    }

    pub fn opacity(windows: WindowSet, opacity: Opacity) -> Self {
        Self::new(windows, AppearanceOperation::Opacity { opacity, toggle: false })
    }

    pub fn colors(windows: WindowSet, colors: impl IntoIterator<Item = (ColorName, ColorValue)>) -> Self {
        Self::new(
            windows,
            AppearanceOperation::Colors {
                colors: colors.into_iter().collect(),
                configured: false,
            },
        )
    }

    /// Reset is global in kitty, including the configured colors for new windows.
    pub fn reset_colors() -> Self {
        Self::new(WindowSet::default(), AppearanceOperation::ResetColors)
    }

    pub fn spacing(windows: WindowSet, settings: impl IntoIterator<Item = (SpacingEdge, Option<Finite>)>) -> Self {
        Self::new(
            windows,
            AppearanceOperation::Spacing {
                settings: settings.into_iter().collect(),
                configured: false,
            },
        )
    }

    pub fn background_image(windows: WindowSet, image: BackgroundImage) -> Self {
        Self::new(
            windows,
            AppearanceOperation::BackgroundImage {
                image,
                layout: BackgroundLayout::Configured,
            },
        )
    }
}

#[derive(Debug, Clone)]
pub enum AppearanceOperation {
    Ligatures(LigatureStrategy),
    Opacity {
        opacity: Opacity,
        toggle: bool,
    },
    Colors {
        colors: BTreeMap<ColorName, ColorValue>,
        configured: bool,
    },
    ResetColors,
    Spacing {
        settings: BTreeMap<SpacingEdge, Option<Finite>>,
        configured: bool,
    },
    BackgroundImage {
        image: BackgroundImage,
        layout: BackgroundLayout,
    },
}
impl Command for Appearance {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.windows.payload();
        let name = match &self.operation {
            AppearanceOperation::Ligatures(strategy) => {
                p["strategy"] = json!(strategy);
                "disable-ligatures"
            }
            AppearanceOperation::Opacity { opacity, toggle } => {
                p["opacity"] = json!(opacity);
                p["toggle"] = json!(toggle);
                "set-background-opacity"
            }
            AppearanceOperation::Colors { colors, configured } => {
                p["colors"] = json!(colors);
                p["configured"] = json!(configured);
                p["reset"] = json!(false);
                "set-colors"
            }
            AppearanceOperation::ResetColors => {
                p["colors"] = json!({});
                p["configured"] = json!(true);
                p["all"] = json!(true);
                p["reset"] = json!(true);
                "set-colors"
            }
            AppearanceOperation::Spacing { settings, configured } => {
                p["settings"] = json!(settings);
                p["configured"] = json!(configured);
                "set-spacing"
            }
            AppearanceOperation::BackgroundImage { image, layout } => {
                let data = match image {
                    BackgroundImage::Remove { .. } => "-".into(),
                    BackgroundImage::Index(n) => format!("index:{n}"),
                    BackgroundImage::Increment(n) => format!("index:{n:+}"),
                };
                // This command's window match field is `match`, not match_window.
                p.as_object_mut().unwrap().remove("match_window");
                p["match"] = json!(self.windows.match_window);
                p["data"] = json!(data);
                p["layout"] = json!(layout);
                p["configured"] = json!(matches!(image, BackgroundImage::Remove { configured: true }));
                "set-background-image"
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
impl CanSubmitWithoutReply for Appearance {}

/// Font size acts on active/all **OS** windows; no match expression exists.
#[derive(Debug, Clone, bon::Builder)]
pub struct SetFontSize {
    pub change: FontChange,
    #[builder(default)]
    pub all: bool,
}
impl SetFontSize {
    pub fn new(change: FontChange) -> Self {
        Self { change, all: false }
    }
}
impl Command for SetFontSize {
    type Output = ();
    fn request(&self) -> PlainRequest {
        request(
            "set-font-size",
            json!({"size": self.change.amount(), "all": self.all, "increment_op": self.change.operator().map(|c| c.to_string())}),
        )
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

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct GetColors {
    #[builder(into)]
    pub match_spec: Option<WindowMatch>,
    #[builder(default)]
    pub configured: bool,
}
impl GetColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_window(window: impl Into<WindowMatch>) -> Self {
        Self {
            match_spec: Some(window.into()),
            ..Self::default()
        }
    }
}
impl Command for GetColors {
    type Output = String;
    fn request(&self) -> PlainRequest {
        request("get-colors", json!({"match": self.match_spec, "configured": self.configured}))
    }
    fn class(&self) -> Class {
        Class::Observation
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<String, DecodeError> {
        crate::decode_string(data)
    }
}
