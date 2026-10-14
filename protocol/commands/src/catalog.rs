//! Audited capability metadata, separate from a selected support matrix.
//!
//! `earliest_documented` is release evidence for that capability, not a tested
//! global floor. Unknown earliest releases stay unknown; no command rewrites
//! the envelope version merely because it was observed on a newer server.
pub const STABLE_REVISION: &str = "b2cb871254fbdac19a085188813c8dfb258b95cd";
pub const HEAD_REVISION: &str = "5bf3c2dc676efd5c4b4526eb135c922701565707";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    TypedFinite,
    FiniteSubsetUploadDeferred,
    UploadDeferred,
    DeprecatedRaw,
    HeadOnlyRaw,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Terminal,
    Deferred,
    Submission,
    NestedDispatch,
    UploadConversation,
}
#[derive(Debug, Clone, Copy)]
pub struct Capability {
    pub name: &'static str,
    pub coverage: Coverage,
    pub completion: Completion,
    /// None means unknown, NOT supported since any arbitrarily old version.
    pub earliest_documented: Option<[u32; 3]>,
    pub source_revision: &'static str,
    pub notes: &'static str,
}
macro_rules! stable {
    ($name:literal, $coverage:ident, $completion:ident, $note:literal) => {
        Capability {
            name: $name,
            coverage: Coverage::$coverage,
            completion: Completion::$completion,
            earliest_documented: None,
            source_revision: STABLE_REVISION,
            notes: $note,
        }
    };
}
pub const CAPABILITIES: &[Capability] = &[
    stable!(
        "action",
        TypedFinite,
        NestedDispatch,
        "Known typed grammar plus raw string; all retire; no downstream completion/replay inference"
    ),
    stable!(
        "close-tab",
        TypedFinite,
        Terminal,
        "Closure request, not child exit"
    ),
    stable!(
        "close-window",
        TypedFinite,
        Terminal,
        "Mark close, not observed disappearance"
    ),
    stable!(
        "create-marker",
        TypedFinite,
        Terminal,
        "Marker pattern pairs or server-loaded function"
    ),
    stable!(
        "detach-tab",
        TypedFinite,
        Terminal,
        "Target tab selects its containing OS window"
    ),
    stable!(
        "detach-window",
        TypedFinite,
        Terminal,
        "Stable target_tab new/empty/match; HEAD extensions not exposed"
    ),
    stable!(
        "disable-ligatures",
        TypedFinite,
        Terminal,
        "never/always/cursor"
    ),
    stable!(
        "env",
        TypedFinite,
        Terminal,
        "Dictionary; future-child environment, deletion NAME="
    ),
    stable!("focus-tab", TypedFinite, Terminal, "First matching tab"),
    stable!(
        "focus-window",
        TypedFinite,
        Terminal,
        "First matching window"
    ),
    stable!(
        "get-colors",
        TypedFinite,
        Terminal,
        "Conf-format string, not JSON map"
    ),
    stable!(
        "get-text",
        TypedFinite,
        Terminal,
        "Nine stable extents; alternates documented since 0.48; optional selection-clearing effect"
    ),
    stable!(
        "goto-layout",
        TypedFinite,
        Terminal,
        "Open layout options/prefix escape"
    ),
    stable!(
        "kitten",
        TypedFinite,
        NestedDispatch,
        "First match; absent/string/bool; installed/custom/UI unknown, retire"
    ),
    stable!(
        "last-used-layout",
        TypedFinite,
        Terminal,
        "Switch previous layout"
    ),
    stable!(
        "launch",
        TypedFinite,
        Terminal,
        "Immediate decimal-ID string or zero; LaunchWait is deferred exit string; no upload"
    ),
    stable!(
        "load-config",
        TypedFinite,
        Terminal,
        "Server paths and list overrides; broad interpretation invalidation"
    ),
    stable!(
        "ls",
        TypedFinite,
        Terminal,
        "JSON string listing; separate session-text profile"
    ),
    stable!(
        "new-window",
        DeprecatedRaw,
        Terminal,
        "Use launch; not legacy serializer"
    ),
    stable!("remove-marker", TypedFinite, Terminal, "Remove marker"),
    stable!(
        "resize-os-window",
        TypedFinite,
        Terminal,
        "WM-dependent resize/state/panel request"
    ),
    stable!(
        "resize-window",
        TypedFinite,
        Terminal,
        "Integer increment and axis; bool/absent/layout-refusal string, not automatically ok:false"
    ),
    stable!(
        "run",
        UploadDeferred,
        UploadConversation,
        "RunOutput decoder only; empty input also requires shared accumulator exclusion"
    ),
    Capability {
        name: "screenshot",
        coverage: Coverage::TypedFinite,
        completion: Completion::Deferred,
        earliest_documented: Some([0, 49, 0]),
        source_revision: STABLE_REVISION,
        notes: "Separate PNG-base64 and server-save path profiles; visible target only",
    },
    stable!(
        "scroll-window",
        TypedFinite,
        Terminal,
        "Amount pair; fractional lines/pages, integral prompts/unscroll"
    ),
    stable!(
        "select-window",
        TypedFinite,
        Deferred,
        "Required selected window integer; no selection is remote failure"
    ),
    stable!(
        "send-key",
        TypedFinite,
        Submission,
        "Keys list; no delivered-key proof"
    ),
    stable!(
        "send-text",
        FiniteSubsetUploadDeferred,
        Submission,
        "One literal UTF-8 submission; no file/stdin/broadcast conversation"
    ),
    stable!(
        "set-background-image",
        FiniteSubsetUploadDeferred,
        Terminal,
        "Removal/index only; PNG upload deferred"
    ),
    stable!(
        "set-background-opacity",
        TypedFinite,
        Terminal,
        "dynamic_background_opacity prerequisite"
    ),
    stable!(
        "set-colors",
        TypedFinite,
        Terminal,
        "RGB/null and transparent-background string; reset implies all/configured"
    ),
    stable!(
        "set-enabled-layouts",
        TypedFinite,
        Terminal,
        "List layouts and configured default"
    ),
    stable!(
        "set-font-size",
        TypedFinite,
        Terminal,
        "OS-window scope, size/all/operator, zero reset"
    ),
    Capability {
        name: "set-os-window-title",
        coverage: Coverage::TypedFinite,
        completion: Completion::Terminal,
        earliest_documented: Some([0, 49, 0]),
        source_revision: STABLE_REVISION,
        notes: "Containing OS windows; optional title restores child",
    },
    stable!(
        "set-spacing",
        TypedFinite,
        Terminal,
        "Canonical edge keys; numeric/null"
    ),
    stable!(
        "set-tab-color",
        TypedFinite,
        Terminal,
        "Four known keys and RGB/null"
    ),
    stable!(
        "set-tab-title",
        TypedFinite,
        Terminal,
        "Empty restores child title"
    ),
    stable!(
        "set-user-vars",
        TypedFinite,
        Terminal,
        "Required string result even for assignments; empty list queries"
    ),
    stable!(
        "set-window-logo",
        FiniteSubsetUploadDeferred,
        Terminal,
        "Removal only; PNG upload deferred"
    ),
    stable!(
        "set-window-title",
        TypedFinite,
        Terminal,
        "Permanent/temporary; missing restores child title"
    ),
    stable!(
        "signal-child",
        TypedFinite,
        Terminal,
        "Foreground signal, not exit completion"
    ),
    Capability {
        name: "resize-window-edge",
        coverage: Coverage::HeadOnlyRaw,
        completion: Completion::Terminal,
        earliest_documented: None,
        source_revision: HEAD_REVISION,
        notes: "Unreleased 0.50dev; not a stable typed contract",
    },
];
pub fn capability(name: &str) -> Option<&'static Capability> {
    CAPABILITIES.iter().find(|c| c.name == name)
}
