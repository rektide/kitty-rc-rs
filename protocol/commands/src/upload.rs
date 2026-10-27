//! Audited upload metadata. Sources and I/O belong to execution; PNG inputs
//! are already encoded images, without implicit filesystem reads/conversion.
use crate::appearance::{BackgroundLayout, LogoPosition};
use crate::process::RunOutput;
use crate::targets::{Assignment, Opacity, WindowMatch, WindowTarget};
use crate::{Class, Command, DecodeError, Mode, request};
use kitty_rc_schema::PlainRequest;
use serde_json::{Value, json};

/// One server-global accumulator per command, not per socket or stream ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UploadKey {
    BackgroundImage,
    WindowLogo,
    Run,
}
impl UploadKey {
    pub fn command(self) -> &'static str {
        match self {
            Self::BackgroundImage => "set-background-image",
            Self::WindowLogo => "set-window-logo",
            Self::Run => "run",
        }
    }
    pub fn raw_chunk_bytes(self) -> usize {
        match self {
            Self::BackgroundImage => 512,
            Self::WindowLogo => 2048,
            Self::Run => 4096,
        }
    }
    pub fn requires_png(self) -> bool {
        self != Self::Run
    }
}
mod sealed {
    pub trait Sealed {}
}
/// Only these independently audited profiles select this conversation.
pub trait UploadCommand: Command + sealed::Sealed {
    fn upload_key(&self) -> UploadKey;
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct BackgroundPng {
    #[builder(into)]
    pub match_spec: Option<WindowMatch>,
    #[builder(default)]
    pub all: bool,
    #[builder(default)]
    pub configured: bool,
    #[builder(default)]
    pub layout: BackgroundLayout,
}
impl BackgroundPng {
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
impl Command for BackgroundPng {
    type Output = ();
    fn request(&self) -> PlainRequest {
        request(
            "set-background-image",
            json!({"match":self.match_spec,"all":self.all,"configured":self.configured,"layout":self.layout}),
        )
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn is_upload(&self) -> bool {
        true
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl sealed::Sealed for BackgroundPng {}
impl UploadCommand for BackgroundPng {
    fn upload_key(&self) -> UploadKey {
        UploadKey::BackgroundImage
    }
}

#[derive(Debug, Clone, Default, bon::Builder)]
pub struct LogoPng {
    #[builder(default)]
    pub target: WindowTarget,
    #[builder(default)]
    pub position: LogoPosition,
    /// None means native configured alpha (-1).
    pub alpha: Option<Opacity>,
}
impl LogoPng {
    pub fn new(target: WindowTarget) -> Self {
        Self {
            target,
            ..Self::default()
        }
    }
}
impl Command for LogoPng {
    type Output = ();
    fn request(&self) -> PlainRequest {
        let mut p = self.target.payload();
        p["position"] = json!(self.position);
        p["alpha"] = self.alpha.map_or(json!(-1.0), |a| json!(a));
        request("set-window-logo", p)
    }
    fn class(&self) -> Class {
        Class::Short
    }
    fn is_upload(&self) -> bool {
        true
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl sealed::Sealed for LogoPng {}
impl UploadCommand for LogoPng {
    fn upload_key(&self) -> UploadKey {
        UploadKey::WindowLogo
    }
}

/// Buffered stdin followed by one async result after child exit, NOT live output.
/// Even empty stdin enters the same server accumulator and coordination lane.
#[derive(Debug, Clone, bon::Builder)]
pub struct Run {
    pub cmdline: Vec<String>,
    #[builder(default)]
    pub env: Vec<Assignment>,
    #[builder(default)]
    pub allow_remote_control: bool,
    #[builder(default)]
    pub remote_control_passwords: Vec<String>,
}
impl Run {
    pub fn new(cmdline: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            cmdline: cmdline.into_iter().map(Into::into).collect(),
            env: Vec::new(),
            allow_remote_control: false,
            remote_control_passwords: Vec::new(),
        }
    }
}
impl Command for Run {
    type Output = RunOutput;
    fn request(&self) -> PlainRequest {
        request(
            "run",
            json!({"cmdline":self.cmdline,"env":self.env.iter().map(Assignment::as_argument).collect::<Vec<_>>(),"allow_remote_control":self.allow_remote_control,"remote_control_password":self.remote_control_passwords}),
        )
    }
    fn class(&self) -> Class {
        Class::LongWait
    }
    fn mode(&self) -> Mode {
        Mode::Deferred
    }
    fn is_upload(&self) -> bool {
        true
    }
    fn reusable(&self) -> bool {
        true
    }
    fn decode(data: Option<Value>) -> Result<RunOutput, DecodeError> {
        RunOutput::decode(data)
    }
}
impl sealed::Sealed for Run {}
impl UploadCommand for Run {
    fn upload_key(&self) -> UploadKey {
        UploadKey::Run
    }
}

pub trait SuppressibleUpload: UploadCommand {}
impl SuppressibleUpload for BackgroundPng {}
impl SuppressibleUpload for LogoPng {}
/// Suppresses only terminal results; the first nonempty handshake remains.
/// Run deliberately does NOT implement SuppressibleUpload.
///
/// ```compile_fail
/// use kitty_rc_commands::upload::{Run, UploadSubmission};
/// let run = Run::builder().cmdline(vec!["cat".into()]).build();
/// let unsupported = UploadSubmission(run);
/// ```
#[derive(Debug, Clone)]
pub struct UploadSubmission<C: SuppressibleUpload>(pub C);
impl<C: SuppressibleUpload> Command for UploadSubmission<C> {
    type Output = ();
    fn request(&self) -> PlainRequest {
        self.0.request()
    }
    fn class(&self) -> Class {
        self.0.class()
    }
    fn mode(&self) -> Mode {
        Mode::SubmissionOnly
    }
    fn is_upload(&self) -> bool {
        true
    }
    fn decode(_: Option<Value>) -> Result<(), DecodeError> {
        Ok(())
    }
}
impl<C: SuppressibleUpload> sealed::Sealed for UploadSubmission<C> {}
impl<C: SuppressibleUpload> UploadCommand for UploadSubmission<C> {
    fn upload_key(&self) -> UploadKey {
        self.0.upload_key()
    }
}
