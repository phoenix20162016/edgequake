//! HTTP `Range` header parsing for binary downloads (RFC 9110 §14).
//!
//! Lets the browser's PDF engine fetch only the byte ranges it needs
//! (first page, then pages on demand) instead of the whole file.
//!
//! Only a single `bytes=` range is honoured. A missing `Range` header still
//! serves the whole body as `200`. A present but unusable header (multipart,
//! non-`bytes` unit, malformed syntax, last < first) is `416`.

/// Outcome of evaluating a `Range` header against a representation length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    /// No usable range: serve the whole body with `200`.
    Full,
    /// Inclusive byte window `start..=end`: serve with `206`.
    Partial { start: u64, end: u64 },
    /// Range starts beyond the end of the body, or the header is unusable: `416`.
    Unsatisfiable,
}

/// Matches pdf.js `rangeChunkSize` so an aborted probe costs at most one window.
pub const PDF_RANGE_CHUNK: u64 = 131_072;

/// How the download handler should emit bytes for an evaluated range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfBodyPlan {
    /// `200` with an empty body (`Content-Length: 0`).
    Empty,
    /// Stream inclusive `start..=end` (as `200` when the request had no Range).
    Stream { start: u64, end: u64 },
    /// `416` with `Content-Range: bytes */total`.
    Unsatisfiable,
}

/// Evaluate an optional `Range` header value against a body of `len` bytes.
pub fn evaluate_range(header: Option<&str>, len: u64) -> ByteRange {
    let Some(raw) = header else {
        return ByteRange::Full;
    };
    if len == 0 {
        return ByteRange::Full;
    }
    let Some(spec) = raw.trim().strip_prefix("bytes=") else {
        return ByteRange::Unsatisfiable;
    };
    // Multi-range requests are optional to support; refuse rather than dump the file.
    if spec.contains(',') {
        return ByteRange::Unsatisfiable;
    }
    let Some((first, last)) = spec.split_once('-') else {
        return ByteRange::Unsatisfiable;
    };
    let (first, last) = (first.trim(), last.trim());

    match (first.is_empty(), last.is_empty()) {
        // "-N": the final N bytes.
        (true, false) => match last.parse::<u64>() {
            Ok(0) => ByteRange::Unsatisfiable,
            Ok(n) => ByteRange::Partial {
                start: len.saturating_sub(n),
                end: len - 1,
            },
            Err(_) => ByteRange::Unsatisfiable,
        },
        // "A-" or "A-B".
        (false, _) => {
            let Ok(start) = first.parse::<u64>() else {
                return ByteRange::Unsatisfiable;
            };
            let end = if last.is_empty() {
                len - 1
            } else {
                match last.parse::<u64>() {
                    Ok(e) => e.min(len - 1),
                    Err(_) => return ByteRange::Unsatisfiable,
                }
            };
            if start >= len || end < start {
                ByteRange::Unsatisfiable
            } else {
                ByteRange::Partial { start, end }
            }
        }
        (true, true) => ByteRange::Unsatisfiable,
    }
}

/// Map an evaluated range onto a body plan.
pub fn pdf_body_plan(range: ByteRange, total: u64) -> PdfBodyPlan {
    match range {
        ByteRange::Unsatisfiable => PdfBodyPlan::Unsatisfiable,
        ByteRange::Full if total == 0 => PdfBodyPlan::Empty,
        ByteRange::Full => PdfBodyPlan::Stream {
            start: 0,
            end: total - 1,
        },
        ByteRange::Partial { start, end } => PdfBodyPlan::Stream { start, end },
    }
}

/// Inclusive windows of at most [`PDF_RANGE_CHUNK`] covering `start..=end`.
pub fn range_windows(start: u64, end: u64) -> Vec<(u64, u64)> {
    if start > end {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut offset = start;
    loop {
        let chunk_end = offset.saturating_add(PDF_RANGE_CHUNK - 1).min(end);
        out.push((offset, chunk_end));
        if chunk_end >= end {
            break;
        }
        offset = chunk_end + 1;
    }
    out
}

/// `Content-Disposition` that cannot poison the header (quotes, CR/LF, non-ASCII).
pub fn pdf_content_disposition(filename: &str) -> String {
    let cleaned: String = filename
        .chars()
        .filter(|c| *c != '"' && *c != '\\' && *c != '\r' && *c != '\n')
        .collect();
    let ascii: String = cleaned
        .chars()
        .map(|c| {
            if c.is_ascii() && !c.is_ascii_control() && c != ';' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let ascii = if ascii.trim().is_empty() {
        "document.pdf"
    } else {
        ascii.as_str()
    };
    let ascii_only = cleaned
        .chars()
        .all(|c| c.is_ascii() && !c.is_ascii_control());
    if ascii_only {
        format!("inline; filename=\"{ascii}\"")
    } else {
        let encoded = urlencoding::encode(&cleaned);
        format!("inline; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_header_is_full() {
        assert_eq!(evaluate_range(None, 100), ByteRange::Full);
    }

    #[test]
    fn closed_range() {
        assert_eq!(
            evaluate_range(Some("bytes=0-9"), 100),
            ByteRange::Partial { start: 0, end: 9 }
        );
    }

    #[test]
    fn single_byte_range() {
        assert_eq!(
            evaluate_range(Some("bytes=0-0"), 100),
            ByteRange::Partial { start: 0, end: 0 }
        );
    }

    #[test]
    fn open_ended_range_runs_to_eof() {
        assert_eq!(
            evaluate_range(Some("bytes=90-"), 100),
            ByteRange::Partial { start: 90, end: 99 }
        );
    }

    #[test]
    fn end_is_clamped_to_length() {
        assert_eq!(
            evaluate_range(Some("bytes=50-5000"), 100),
            ByteRange::Partial { start: 50, end: 99 }
        );
    }

    #[test]
    fn suffix_range_returns_tail() {
        assert_eq!(
            evaluate_range(Some("bytes=-10"), 100),
            ByteRange::Partial { start: 90, end: 99 }
        );
        // Suffix longer than the body returns the whole body as 206.
        assert_eq!(
            evaluate_range(Some("bytes=-500"), 100),
            ByteRange::Partial { start: 0, end: 99 }
        );
    }

    #[test]
    fn start_past_eof_is_unsatisfiable() {
        assert_eq!(
            evaluate_range(Some("bytes=100-"), 100),
            ByteRange::Unsatisfiable
        );
        assert_eq!(
            evaluate_range(Some("bytes=-0"), 100),
            ByteRange::Unsatisfiable
        );
    }

    #[test]
    fn unusable_headers_are_unsatisfiable() {
        for h in [
            "items=0-9",
            "bytes=0-9,20-29",
            "bytes=abc-",
            "bytes=9-0",
            "bytes=-",
            "bytes",
            "",
            "Bytes=0-9",
            "bytes =0-9",
        ] {
            assert_eq!(
                evaluate_range(Some(h), 100),
                ByteRange::Unsatisfiable,
                "{h:?}"
            );
        }
    }

    #[test]
    fn empty_body_is_always_full() {
        assert_eq!(evaluate_range(Some("bytes=0-9"), 0), ByteRange::Full);
        assert_eq!(pdf_body_plan(ByteRange::Full, 0), PdfBodyPlan::Empty);
    }

    #[test]
    fn body_plan_streams_full_and_partial() {
        assert_eq!(
            pdf_body_plan(ByteRange::Full, 100),
            PdfBodyPlan::Stream { start: 0, end: 99 }
        );
        assert_eq!(
            pdf_body_plan(ByteRange::Partial { start: 10, end: 19 }, 100),
            PdfBodyPlan::Stream { start: 10, end: 19 }
        );
        assert_eq!(
            pdf_body_plan(ByteRange::Unsatisfiable, 100),
            PdfBodyPlan::Unsatisfiable
        );
    }

    #[test]
    fn range_windows_split_on_chunk_size() {
        assert_eq!(range_windows(0, 0), vec![(0, 0)]);
        assert_eq!(
            range_windows(0, PDF_RANGE_CHUNK - 1),
            vec![(0, PDF_RANGE_CHUNK - 1)]
        );
        assert_eq!(
            range_windows(0, PDF_RANGE_CHUNK),
            vec![(0, PDF_RANGE_CHUNK - 1), (PDF_RANGE_CHUNK, PDF_RANGE_CHUNK)]
        );
        assert_eq!(range_windows(10, 20), vec![(10, 20)]);
        assert!(range_windows(5, 4).is_empty());
    }

    #[test]
    fn disposition_strips_quotes_and_newlines() {
        let d = pdf_content_disposition("a\"b\nc.pdf");
        assert_eq!(d, "inline; filename=\"abc.pdf\"");
    }

    #[test]
    fn disposition_uses_filename_star_for_non_ascii() {
        let d = pdf_content_disposition("résumé.pdf");
        assert!(d.contains("filename*=UTF-8''"));
        assert!(d.contains("inline; filename="));
    }

    #[test]
    fn disposition_falls_back_when_empty() {
        assert_eq!(
            pdf_content_disposition("\n\r"),
            "inline; filename=\"document.pdf\""
        );
    }
}
