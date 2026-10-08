//! Typed MCP cursors `{object}:{offset}` (SPEC-162 R14).

use serde_json::Value;

use super::errors::{eq_error, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorObject {
    Entities,
    Relationships,
    Chunks,
    Bytes,
}

impl CursorObject {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entities => "entities",
            Self::Relationships => "relationships",
            Self::Chunks => "chunks",
            Self::Bytes => "bytes",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "entities" => Some(Self::Entities),
            "relationships" => Some(Self::Relationships),
            "chunks" => Some(Self::Chunks),
            "bytes" => Some(Self::Bytes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub object: CursorObject,
    pub offset: usize,
}

impl Cursor {
    pub fn encode(&self) -> String {
        format!("{}:{}", self.object.as_str(), self.offset)
    }

    pub fn parse(raw: &str) -> Result<Self, Value> {
        let Some((obj, off)) = raw.split_once(':') else {
            return Err(eq_error(
                ErrorCode::InvalidId,
                format!("invalid cursor: {raw}"),
                None,
            ));
        };
        let object = CursorObject::parse(obj).ok_or_else(|| {
            eq_error(
                ErrorCode::InvalidId,
                format!("invalid cursor object: {obj}"),
                None,
            )
        })?;
        let offset: usize = off.parse().map_err(|_| {
            eq_error(
                ErrorCode::InvalidId,
                format!("invalid cursor offset: {off}"),
                None,
            )
        })?;
        Ok(Self { object, offset })
    }

    pub fn require_object(raw: Option<&str>, expected: CursorObject) -> Result<usize, Value> {
        let Some(raw) = raw.filter(|s| !s.is_empty()) else {
            return Ok(0);
        };
        let c = Self::parse(raw)?;
        if c.object != expected {
            return Err(eq_error(
                ErrorCode::InvalidId,
                format!(
                    "cursor object {} does not match {}",
                    c.object.as_str(),
                    expected.as_str()
                ),
                None,
            ));
        }
        Ok(c.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let c = Cursor {
            object: CursorObject::Entities,
            offset: 16,
        };
        assert_eq!(c.encode(), "entities:16");
        let p = Cursor::parse("entities:16").unwrap();
        assert_eq!(p, c);
    }

    #[test]
    fn rejects_wrong_object() {
        let err = Cursor::require_object(Some("chunks:3"), CursorObject::Entities);
        assert!(err.is_err());
    }
}
