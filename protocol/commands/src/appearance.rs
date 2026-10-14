//! Finite appearance payloads, not client-side config-file readers or image
//! uploads. Scope fields follow each native command rather than a fake universal
//! window selector.
use crate::targets::{Finite, InvalidValue, Opacity, Rgb, WindowSet};
use crate::{CanSubmitWithoutReply, Class, Command, DecodeError, request};
use kitty_rc_schema::PlainRequest;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

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
        if !amount.is_finite()
            || amount < 0.0
            || (matches!(operator, Some('*' | '/')) && amount == 0.0)
        {
            return Err(InvalidValue);
        }
        Ok(Self { amount, operator })
    }
    pub fn reset() -> Self {
        Self {
            amount: 0.0,
            operator: None,
        }
    }
    pub fn amount(self) -> f64 {
        self.amount
    }
    pub fn operator(self) -> Option<char> {
        self.operator
    }
    pub fn argument(self) -> String {
        format!(
            "{}{}",
            self.operator.map_or_else(String::new, |c| c.to_string()),
            self.amount
        )
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

#[derive(Debug, Clone)]
pub struct Appearance {
    pub windows: WindowSet,
    pub operation: AppearanceOperation,
}
#[derive(Debug, Clone)]
pub enum AppearanceOperation {
    Ligatures(LigatureStrategy),
    Opacity {
        opacity: Opacity,
        toggle: bool,
    },
    Colors {
        colors: BTreeMap<String, ColorValue>,
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
            AppearanceOperation::Spacing {
                settings,
                configured,
            } => {
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
                p["configured"] = json!(matches!(
                    image,
                    BackgroundImage::Remove { configured: true }
                ));
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
#[builder(on(String, into))]
pub struct GetColors {
    pub match_spec: Option<String>,
    #[builder(default)]
    pub configured: bool,
}
impl Command for GetColors {
    type Output = String;
    fn request(&self) -> PlainRequest {
        request(
            "get-colors",
            json!({"match": self.match_spec, "configured": self.configured}),
        )
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
