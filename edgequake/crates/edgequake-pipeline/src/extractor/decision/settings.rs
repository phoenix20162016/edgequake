//! Settings of the decision mode (SPEC-160).
//!
//! Three layers feed one run:
//!
//! | Layer      | Source                                   | Holds                         |
//! |------------|------------------------------------------|-------------------------------|
//! | Server     | `EDGEQUAKE_DECISION_*` environment       | backend, URL, key, limits     |
//! | Workspace  | workspace metadata `decision_*`          | model, pack size, gate preset |
//! | Document   | task metadata `decision_gate_preset`     | gate preset                   |
//!
//! Everything here is pure. [`DecisionSettings::from_lookup`] takes a lookup
//! function, so tests need no process environment. A bad value is an error,
//! never a silent default (LAW-160-4).

use std::collections::HashMap;

use serde_json::Value;

use super::backend::BackendKind;
use super::error::DecisionError;
use super::gate::{GatePreset, META_DECISION_GATE_PRESET};

pub const DECISION_ENABLED_ENV: &str = "EDGEQUAKE_DECISION_ENABLED";
pub const DECISION_BACKEND_ENV: &str = "EDGEQUAKE_DECISION_BACKEND";
pub const DECISION_BASE_URL_ENV: &str = "EDGEQUAKE_DECISION_BASE_URL";
pub const DECISION_API_KEY_ENV: &str = "EDGEQUAKE_DECISION_API_KEY";
pub const DECISION_MODEL_ENV: &str = "EDGEQUAKE_DECISION_MODEL";
pub const DECISION_PACK_SIZE_ENV: &str = "EDGEQUAKE_DECISION_PACK_SIZE";
pub const DECISION_GATE_PRESET_ENV: &str = "EDGEQUAKE_DECISION_GATE_PRESET";
pub const DECISION_TIMEOUT_ENV: &str = "EDGEQUAKE_DECISION_TIMEOUT_SECS";
pub const DECISION_KEEP_ALIVE_ENV: &str = "EDGEQUAKE_DECISION_KEEP_ALIVE";
pub const DECISION_CACHE_TTL_ENV: &str = "EDGEQUAKE_DECISION_CACHE_TTL_DAYS";
pub const DECISION_CACHE_MAX_ROWS_ENV: &str = "EDGEQUAKE_DECISION_CACHE_MAX_ROWS";

/// Workspace metadata keys.
pub const META_DECISION_MODEL: &str = "decision_model";
pub const META_DECISION_PACK_SIZE: &str = "decision_pack_size";
pub const META_DECISION_ENABLED: &str = "decision_enabled";

/// Defaults.
pub const DEFAULT_DECISION_MODEL: &str = "tev1:0.8b";
/// Well-known local Ollama (`ollama serve`). Decision does not follow `OLLAMA_HOST`.
pub const DEFAULT_DECISION_BASE_URL: &str = "http://localhost:11434";
pub const DEFAULT_PACK_SIZE: usize = 4;
pub const MIN_PACK_SIZE: usize = 1;
pub const MAX_PACK_SIZE: usize = 16;
pub const DEFAULT_TIMEOUT_SECS: u64 = 600;
pub const DEFAULT_CACHE_TTL_DAYS: u32 = 30;
pub const DEFAULT_CACHE_MAX_ROWS: u64 = 200_000;
const DEFAULT_BASE_URL: &str = DEFAULT_DECISION_BASE_URL;
const DEFAULT_KEEP_ALIVE: &str = "30m";
const MAX_MODEL_LEN: usize = 200;

/// The `openai_logprobs` adapter is deferred (spec 13, W6): no Tev1 GGUF exists to
/// prove it, so the boot check refuses the value instead of failing on first upload.
fn reject_unbuilt_backend(kind: BackendKind, raw: &str) -> Result<(), DecisionError> {
    match kind {
        BackendKind::OllamaSystemOne => Ok(()),
        BackendKind::OpenAiLogprobs => Err(DecisionError::Config(format!(
            "{DECISION_BACKEND_ENV}='{raw}' is not available in this release. \
             Use ollama_system_one."
        ))),
    }
}

/// Words that clear a workspace override.
const INHERIT_WORDS: [&str; 3] = ["", "inherit", "none"];

/// How `EDGEQUAKE_DECISION_ENABLED` combines with a workspace opt-in.
///
/// | Env         | Workspace `decision_enabled` | Result     |
/// |-------------|------------------------------|------------|
/// | unset / `1` | any                          | forced on  |
/// | `0`         | any                          | locked     |
/// | `workspace` | true                         | active     |
/// | `workspace` | false / absent               | inactive   |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionGate {
    /// Operator locked the mode off. Workspaces cannot turn it on.
    Locked,
    /// Unset env: each workspace opts in with `decision_enabled`.
    WorkspaceChoice,
    /// Operator forced the mode on for every workspace.
    ForcedOn,
}

impl DecisionGate {
    /// Parse the env value. Empty and missing default to [`Self::ForcedOn`].
    pub fn from_raw(raw: Option<&str>) -> Result<Self, DecisionError> {
        match raw.map(str::trim).filter(|s| !s.is_empty()) {
            None => Ok(Self::ForcedOn),
            Some(v) => match v.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Ok(Self::ForcedOn),
                "0" | "false" | "no" | "off" => Ok(Self::Locked),
                "workspace" | "optin" | "auto" => Ok(Self::WorkspaceChoice),
                other => Err(DecisionError::Config(format!(
                    "{DECISION_ENABLED_ENV}='{other}' is not a boolean. Use 1, 0, workspace, or leave it unset."
                ))),
            },
        }
    }

    /// Combine with the workspace opt-in into a typed activation.
    pub fn activation(self, workspace_enabled: bool) -> DecisionActivation {
        DecisionActivation::of(self, workspace_enabled)
    }
}

/// What a workspace can do with the decision engine right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionActivation {
    Locked,
    Inactive,
    Active,
    Forced,
}

impl DecisionActivation {
    pub fn of(gate: DecisionGate, workspace_enabled: bool) -> Self {
        match gate {
            DecisionGate::Locked => Self::Locked,
            DecisionGate::ForcedOn => Self::Forced,
            DecisionGate::WorkspaceChoice => {
                if workspace_enabled {
                    Self::Active
                } else {
                    Self::Inactive
                }
            }
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Locked => "locked",
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Forced => "forced",
        }
    }

    /// True when a decision run is allowed to start (backend still has to be ready).
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Active | Self::Forced)
    }

    /// The typed refusal, if a run must not start.
    pub fn refusal(self) -> Option<DecisionError> {
        match self {
            Self::Locked => Some(DecisionError::Disabled),
            Self::Inactive => Some(DecisionError::NotActivated),
            Self::Active | Self::Forced => None,
        }
    }
}

/// Server-level settings, validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionSettings {
    pub gate: DecisionGate,
    pub backend: BackendKind,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub pack_size: usize,
    pub gate_preset: GatePreset,
    pub timeout_secs: u64,
    pub keep_alive: String,
    pub cache_ttl_days: u32,
    pub cache_max_rows: u64,
}

impl Default for DecisionSettings {
    fn default() -> Self {
        Self {
            gate: DecisionGate::ForcedOn,
            backend: BackendKind::OllamaSystemOne,
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: None,
            model: DEFAULT_DECISION_MODEL.to_string(),
            pack_size: DEFAULT_PACK_SIZE,
            gate_preset: GatePreset::default(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            keep_alive: DEFAULT_KEEP_ALIVE.to_string(),
            cache_ttl_days: DEFAULT_CACHE_TTL_DAYS,
            cache_max_rows: DEFAULT_CACHE_MAX_ROWS,
        }
    }
}

fn parse_number<T: std::str::FromStr>(name: &str, raw: &str) -> Result<T, DecisionError> {
    raw.trim().parse::<T>().map_err(|_| {
        DecisionError::Config(format!("{name}='{}' is not a valid number.", raw.trim()))
    })
}

/// Check a pack size against the allowed range.
pub fn check_pack_size(value: i64) -> Result<usize, DecisionError> {
    usize::try_from(value)
        .ok()
        .filter(|v| (MIN_PACK_SIZE..=MAX_PACK_SIZE).contains(v))
        .ok_or_else(|| {
            DecisionError::Config(format!(
                "decision pack size {value} is outside {MIN_PACK_SIZE}-{MAX_PACK_SIZE}."
            ))
        })
}

/// Check a model name: not empty, no spaces or control characters, bounded.
pub fn check_model_name(raw: &str) -> Result<String, DecisionError> {
    let m = raw.trim();
    if m.is_empty()
        || m.len() > MAX_MODEL_LEN
        || m.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(DecisionError::Config(format!(
            "decision model '{m}' is not a valid model name."
        )));
    }
    Ok(m.to_string())
}

/// Check an Ollama `keep_alive` value: a whole number of seconds (`-1` = forever)
/// or a duration such as `30m`, `1h30m`, `90s`. Ollama rejects other forms with a 400.
pub fn check_keep_alive(raw: &str) -> Result<String, DecisionError> {
    let t = raw.trim();
    if is_keep_alive(t) {
        return Ok(t.to_string());
    }
    Err(DecisionError::Config(format!(
        "{DECISION_KEEP_ALIVE_ENV}='{t}' is not valid. Use seconds (300, -1) or a duration (30m, 1h30m)."
    )))
}

fn is_keep_alive(t: &str) -> bool {
    let digits = t.strip_prefix('-').unwrap_or(t);
    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
        return true;
    }
    // Go duration: one or more `<digits><unit>` pairs.
    let mut rest = t;
    let mut pairs = 0;
    while !rest.is_empty() {
        let num_end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(0);
        if num_end == 0 {
            return false;
        }
        rest = &rest[num_end..];
        let Some(unit) = ["ms", "us", "ns", "h", "m", "s"]
            .iter()
            .find(|u| rest.starts_with(**u))
        else {
            return false;
        };
        rest = &rest[unit.len()..];
        pairs += 1;
    }
    pairs > 0
}

fn check_gate_preset(raw: &str) -> Result<GatePreset, DecisionError> {
    GatePreset::parse(raw).ok_or_else(|| {
        DecisionError::Config(format!(
            "decision gate preset '{}' is unknown. Allowed: inherit, strict, balanced, recall.",
            raw.trim()
        ))
    })
}

impl DecisionSettings {
    /// Read `EDGEQUAKE_DECISION_*` from the process environment.
    pub fn from_env() -> Result<Self, DecisionError> {
        Self::from_lookup(&|name| std::env::var(name).ok())
    }

    /// Pure variant of [`Self::from_env`].
    pub fn from_lookup(get: &dyn Fn(&str) -> Option<String>) -> Result<Self, DecisionError> {
        let mut s = Self::default();
        let non_empty = |name: &str| get(name).filter(|v| !v.trim().is_empty());
        s.gate = DecisionGate::from_raw(get(DECISION_ENABLED_ENV).as_deref())?;
        if let Some(v) = non_empty(DECISION_BACKEND_ENV) {
            s.backend = BackendKind::parse(&v).ok_or_else(|| {
                DecisionError::Config(format!(
                    "{DECISION_BACKEND_ENV}='{v}' is unknown. Allowed: ollama_system_one."
                ))
            })?;
            reject_unbuilt_backend(s.backend, &v)?;
        }
        if let Some(v) = non_empty(DECISION_BASE_URL_ENV) {
            s.base_url = normalize_base_url(&v)?;
        } else {
            s.base_url = DEFAULT_DECISION_BASE_URL.to_string();
        }
        s.api_key = non_empty(DECISION_API_KEY_ENV);
        if let Some(v) = non_empty(DECISION_MODEL_ENV) {
            s.model = check_model_name(&v)?;
        }
        if let Some(v) = non_empty(DECISION_PACK_SIZE_ENV) {
            s.pack_size = check_pack_size(parse_number(DECISION_PACK_SIZE_ENV, &v)?)?;
        }
        if let Some(v) = non_empty(DECISION_GATE_PRESET_ENV) {
            s.gate_preset = check_gate_preset(&v)?;
        }
        if let Some(v) = non_empty(DECISION_TIMEOUT_ENV) {
            s.timeout_secs = parse_number::<u64>(DECISION_TIMEOUT_ENV, &v)?.clamp(1, 86_400);
        }
        if let Some(v) = non_empty(DECISION_KEEP_ALIVE_ENV) {
            s.keep_alive = check_keep_alive(&v)?;
        }
        if let Some(v) = non_empty(DECISION_CACHE_TTL_ENV) {
            s.cache_ttl_days = parse_number(DECISION_CACHE_TTL_ENV, &v)?;
        }
        if let Some(v) = non_empty(DECISION_CACHE_MAX_ROWS_ENV) {
            s.cache_max_rows = parse_number(DECISION_CACHE_MAX_ROWS_ENV, &v)?;
        }
        Ok(s)
    }

    /// Apply workspace and document overrides. Override fields win.
    pub fn with_overrides(&self, o: &DecisionOverrides) -> Self {
        let mut s = self.clone();
        if let Some(m) = &o.model {
            s.model = m.clone();
        }
        if let Some(p) = o.pack_size {
            s.pack_size = p;
        }
        if let Some(g) = o.gate_preset {
            s.gate_preset = g;
        }
        s
    }
}

/// `OLLAMA_HOST` may be `host:port` without a scheme. Accept both forms.
fn normalize_base_url(raw: &str) -> Result<String, DecisionError> {
    let t = raw.trim().trim_end_matches('/');
    let with_scheme = if t.contains("://") {
        t.to_string()
    } else {
        format!("http://{t}")
    };
    if !(with_scheme.starts_with("http://") || with_scheme.starts_with("https://")) {
        return Err(DecisionError::Config(format!(
            "decision base URL '{raw}' must start with http:// or https://."
        )));
    }
    Ok(with_scheme)
}

/// Workspace and document overrides, already validated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecisionOverrides {
    pub model: Option<String>,
    pub pack_size: Option<usize>,
    pub gate_preset: Option<GatePreset>,
}

impl DecisionOverrides {
    /// Read workspace metadata. A bad stored value is an error, not a default.
    pub fn from_workspace_metadata(meta: &HashMap<String, Value>) -> Result<Self, DecisionError> {
        let model = match meta.get(META_DECISION_MODEL).and_then(Value::as_str) {
            Some(m) if !is_inherit(m) => Some(check_model_name(m)?),
            _ => None,
        };
        let pack_size = match meta.get(META_DECISION_PACK_SIZE).and_then(Value::as_i64) {
            Some(p) => Some(check_pack_size(p)?),
            None => None,
        };
        let gate_preset = match meta.get(META_DECISION_GATE_PRESET).and_then(Value::as_str) {
            Some(g) if !is_inherit(g) => Some(check_gate_preset(g)?),
            _ => None,
        };
        Ok(Self {
            model,
            pack_size,
            gate_preset,
        })
    }

    /// Layer a document override (only the gate preset) over workspace overrides.
    pub fn with_document(mut self, doc_meta: &Value) -> Result<Self, DecisionError> {
        if let Some(g) = doc_meta
            .get(META_DECISION_GATE_PRESET)
            .and_then(Value::as_str)
            .filter(|g| !is_inherit(g))
        {
            self.gate_preset = Some(check_gate_preset(g)?);
        }
        Ok(self)
    }
}

fn is_inherit(raw: &str) -> bool {
    INHERIT_WORDS.contains(&raw.trim().to_ascii_lowercase().as_str())
}

/// True when the workspace metadata opts the engine on.
pub fn workspace_decision_enabled(meta: &HashMap<String, Value>) -> bool {
    meta.get(META_DECISION_ENABLED).and_then(Value::as_bool) == Some(true)
}

/// A workspace write request for the decision keys.
#[derive(Debug, Clone, Default)]
pub struct DecisionMetadataRequest<'a> {
    pub model: Option<&'a str>,
    /// `0` clears the override. Other values must be in range.
    pub pack_size: Option<i64>,
    pub gate_preset: Option<&'a str>,
    /// Workspace opt-in. `None` leaves the key unchanged.
    pub enabled: Option<bool>,
}

/// Validate a document-level gate preset word. `Ok(None)` means inherit.
pub fn parse_gate_preset_override(raw: Option<&str>) -> Result<Option<GatePreset>, DecisionError> {
    match raw {
        Some(g) if !is_inherit(g) => check_gate_preset(g).map(Some),
        _ => Ok(None),
    }
}

/// Apply a workspace write to metadata (shared by Postgres and in-memory).
///
/// All fields are checked first. On error the metadata stays unchanged.
pub fn apply_decision_metadata(
    meta: &mut HashMap<String, Value>,
    req: &DecisionMetadataRequest<'_>,
) -> Result<(), DecisionError> {
    let model = match req.model {
        Some(m) if is_inherit(m) => Some(None),
        Some(m) => Some(Some(check_model_name(m)?)),
        None => None,
    };
    let pack = match req.pack_size {
        Some(0) => Some(None),
        Some(p) => Some(Some(check_pack_size(p)?)),
        None => None,
    };
    let preset = match req.gate_preset {
        Some(g) if is_inherit(g) => Some(None),
        Some(g) => Some(Some(check_gate_preset(g)?)),
        None => None,
    };
    set_or_clear(meta, META_DECISION_MODEL, model.map(|o| o.map(Value::from)));
    set_or_clear(
        meta,
        META_DECISION_PACK_SIZE,
        pack.map(|o| o.map(Value::from)),
    );
    set_or_clear(
        meta,
        META_DECISION_GATE_PRESET,
        preset.map(|o| o.map(|p| Value::from(p.as_str()))),
    );
    if let Some(on) = req.enabled {
        meta.insert(META_DECISION_ENABLED.to_string(), Value::from(on));
    }
    Ok(())
}

fn set_or_clear(meta: &mut HashMap<String, Value>, key: &str, change: Option<Option<Value>>) {
    match change {
        None => {}
        Some(None) => {
            meta.remove(key);
        }
        Some(Some(v)) => {
            meta.insert(key.to_string(), v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let m: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| m.get(k).cloned()
    }

    // T-160-U61 — keep_alive accepts what Ollama accepts and nothing else.
    #[test]
    fn keep_alive_forms() {
        for ok in ["30m", "1h30m", "90s", "300", "-1", "500ms", " 5m "] {
            assert!(check_keep_alive(ok).is_ok(), "{ok}");
        }
        for bad in ["", "m", "30 minutes", "bogus!", "1x", "--1", "5m5"] {
            assert!(check_keep_alive(bad).is_err(), "{bad}");
        }
        let err = DecisionSettings::from_lookup(&env(&[(DECISION_KEEP_ALIVE_ENV, "soon")]));
        assert_eq!(err.unwrap_err().code(), "invalid_decision_setting");
    }

    // T-160-U42 — defaults when nothing is set; mode off.
    #[test]
    fn defaults_are_safe() {
        let s = DecisionSettings::from_lookup(&env(&[])).unwrap();
        assert_eq!(s.gate, DecisionGate::ForcedOn);
        assert_eq!(s.model, "tev1:0.8b");
        assert_eq!(s.pack_size, 4);
        assert_eq!(s.gate_preset, GatePreset::Balanced);
        assert_eq!(s.base_url, DEFAULT_DECISION_BASE_URL);
    }

    // T-160-U33 — bad values fail instead of defaulting (EC-160-02).
    #[test]
    fn bad_values_fail() {
        for (k, v) in [
            (DECISION_ENABLED_ENV, "maybe"),
            (DECISION_BACKEND_ENV, "gpt"),
            (DECISION_PACK_SIZE_ENV, "0"),
            (DECISION_PACK_SIZE_ENV, "17"),
            (DECISION_PACK_SIZE_ENV, "four"),
            (DECISION_GATE_PRESET_ENV, "loose"),
            (DECISION_MODEL_ENV, "bad model"),
            (DECISION_TIMEOUT_ENV, "x"),
        ] {
            let err = DecisionSettings::from_lookup(&env(&[(k, v)])).unwrap_err();
            assert_eq!(err.code(), "invalid_decision_setting", "{k}={v}");
        }
    }

    // T-160-U43 — Decision uses the local Ollama URL; OLLAMA_HOST is ignored.
    #[test]
    fn base_url_forms() {
        let s = DecisionSettings::from_lookup(&env(&[("OLLAMA_HOST", "10.0.0.5:11434")])).unwrap();
        assert_eq!(s.base_url, DEFAULT_DECISION_BASE_URL);
        let s = DecisionSettings::from_lookup(&env(&[
            ("OLLAMA_HOST", "10.0.0.5:11434"),
            (DECISION_BASE_URL_ENV, "https://gpu.example.com/"),
        ]))
        .unwrap();
        assert_eq!(s.base_url, "https://gpu.example.com");
        assert!(
            DecisionSettings::from_lookup(&env(&[(DECISION_BASE_URL_ENV, "ftp://x")])).is_err()
        );
    }

    // T-160-U93 — a backend that is not built is refused at boot, by name.
    #[test]
    fn unbuilt_backend_is_refused() {
        let err = DecisionSettings::from_lookup(&env(&[(DECISION_BACKEND_ENV, "openai_logprobs")]))
            .unwrap_err();
        assert!(err.to_string().contains("not available in this release"));
        assert!(err.to_string().contains(DECISION_BACKEND_ENV));
    }

    // T-160-U44 — a full valid environment parses.
    #[test]
    fn full_env_parses() {
        let s = DecisionSettings::from_lookup(&env(&[
            (DECISION_ENABLED_ENV, "1"),
            (DECISION_BACKEND_ENV, "ollama"),
            (DECISION_MODEL_ENV, "tev1:4b"),
            (DECISION_PACK_SIZE_ENV, "8"),
            (DECISION_GATE_PRESET_ENV, "strict"),
            (DECISION_API_KEY_ENV, "k"),
            (DECISION_CACHE_TTL_ENV, "7"),
        ]))
        .unwrap();
        assert!(s.gate == DecisionGate::ForcedOn);
        assert_eq!(s.backend, BackendKind::OllamaSystemOne);
        assert_eq!((s.model.as_str(), s.pack_size), ("tev1:4b", 8));
        assert_eq!(s.gate_preset, GatePreset::Strict);
        assert_eq!(s.api_key.as_deref(), Some("k"));
        assert_eq!(s.cache_ttl_days, 7);
    }

    // T-160-U45 — overrides layer: document preset beats workspace preset.
    #[test]
    fn overrides_layering() {
        let mut meta = HashMap::new();
        meta.insert(META_DECISION_MODEL.to_string(), json!("tev1:4b"));
        meta.insert(META_DECISION_PACK_SIZE.to_string(), json!(2));
        meta.insert(META_DECISION_GATE_PRESET.to_string(), json!("strict"));
        let o = DecisionOverrides::from_workspace_metadata(&meta)
            .unwrap()
            .with_document(&json!({ "decision_gate_preset": "recall" }))
            .unwrap();
        let s = DecisionSettings::default().with_overrides(&o);
        assert_eq!((s.model.as_str(), s.pack_size), ("tev1:4b", 2));
        assert_eq!(s.gate_preset, GatePreset::Recall);
        let o2 = DecisionOverrides::from_workspace_metadata(&meta)
            .unwrap()
            .with_document(&json!({ "decision_gate_preset": "inherit" }))
            .unwrap();
        assert_eq!(o2.gate_preset, Some(GatePreset::Strict));
    }

    // T-160-U46 — a bad stored workspace value is reported.
    #[test]
    fn bad_stored_value_is_error() {
        let mut meta = HashMap::new();
        meta.insert(META_DECISION_PACK_SIZE.to_string(), json!(99));
        assert!(DecisionOverrides::from_workspace_metadata(&meta).is_err());
    }

    // T-160-U47 — workspace write: set, clear, reject leaves metadata unchanged.
    #[test]
    fn apply_metadata_set_clear_reject() {
        let mut meta = HashMap::new();
        meta.insert("other".to_string(), json!("keep"));
        apply_decision_metadata(
            &mut meta,
            &DecisionMetadataRequest {
                model: Some("tev1:4b"),
                pack_size: Some(6),
                gate_preset: Some("Strict"),
                enabled: Some(true),
            },
        )
        .unwrap();
        assert_eq!(meta[META_DECISION_MODEL], json!("tev1:4b"));
        assert_eq!(meta[META_DECISION_PACK_SIZE], json!(6));
        assert_eq!(meta[META_DECISION_GATE_PRESET], json!("strict"));
        assert_eq!(meta[META_DECISION_ENABLED], json!(true));

        let err = apply_decision_metadata(
            &mut meta,
            &DecisionMetadataRequest {
                model: Some("clear-me?"),
                pack_size: Some(99),
                gate_preset: None,
                enabled: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code(), "invalid_decision_setting");
        assert_eq!(
            meta[META_DECISION_PACK_SIZE],
            json!(6),
            "unchanged on error"
        );

        apply_decision_metadata(
            &mut meta,
            &DecisionMetadataRequest {
                model: Some("inherit"),
                pack_size: Some(0),
                gate_preset: Some(""),
                enabled: Some(false),
            },
        )
        .unwrap();
        assert_eq!(meta["other"], json!("keep"));
        assert_eq!(meta[META_DECISION_ENABLED], json!(false));
        assert_eq!(meta.len(), 2);
    }

    #[test]
    fn gate_truth_table() {
        let cases = [
            (None, false, DecisionActivation::Forced),
            (None, true, DecisionActivation::Forced),
            (Some("0"), false, DecisionActivation::Locked),
            (Some("0"), true, DecisionActivation::Locked),
            (Some("1"), false, DecisionActivation::Forced),
            (Some("1"), true, DecisionActivation::Forced),
            (Some(""), false, DecisionActivation::Forced),
            (Some("workspace"), false, DecisionActivation::Inactive),
            (Some("workspace"), true, DecisionActivation::Active),
            (Some("false"), true, DecisionActivation::Locked),
            (Some("on"), false, DecisionActivation::Forced),
        ];
        for (raw, ws, want) in cases {
            let got = DecisionGate::from_raw(raw).unwrap().activation(ws);
            assert_eq!(got, want, "env={raw:?} ws={ws}");
        }
        assert!(DecisionGate::from_raw(Some("maybe")).is_err());
        assert_eq!(
            DecisionActivation::Inactive.refusal(),
            Some(DecisionError::NotActivated)
        );
        assert_eq!(
            DecisionActivation::Locked.refusal(),
            Some(DecisionError::Disabled)
        );
        assert!(DecisionActivation::Active.refusal().is_none());
        assert!(DecisionActivation::Forced.is_usable());
    }

    #[test]
    fn model_name_rules() {
        assert!(check_model_name("tev1:0.8b").is_ok());
        assert!(check_model_name("registry/lib/tev1:4b").is_ok());
        assert!(check_model_name("").is_err());
        assert!(check_model_name("a b").is_err());
        assert!(check_model_name(&"x".repeat(201)).is_err());
    }
}
