//! A minimal, zero-dependency JSON reader — just enough to load a cognition
//! *contract* (`intent.json`) from disk.
//!
//! confevo refuses to take on a JSON dependency (it must build green even when the
//! root crate cannot), so this is a compact recursive-descent parser over the
//! subset of JSON that cognition contracts use: objects, arrays, strings, numbers,
//! booleans, and null. It is deliberately small and is **not** a conformant JSON
//! library; it exists only to read the breed contract shape.

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    /// JSON `null`.
    Null,
    /// JSON `true` / `false`.
    Bool(bool),
    /// JSON number (always stored as `f64`).
    Num(f64),
    /// JSON string (escapes already decoded).
    Str(String),
    /// JSON array.
    Arr(Vec<Value>),
    /// JSON object, preserving key order.
    Obj(Vec<(String, Value)>),
}

impl Value {
    /// Borrow this value as a string, if it is one.
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Borrow this value as an array slice, if it is one.
    pub(crate) fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }

    /// Look up `key` in an object value.
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

/// Parse a complete JSON document, rejecting trailing non-whitespace.
pub(crate) fn parse(input: &str) -> Result<Value, String> {
    let mut p = Parser {
        bytes: input.as_bytes(),
        pos: 0,
    };
    p.skip_ws();
    let v = p.value()?;
    p.skip_ws();
    if p.pos != p.bytes.len() {
        return Err(format!("trailing data at byte {}", p.pos));
    }
    Ok(v)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Result<u8, String> {
        self.bytes
            .get(self.pos)
            .copied()
            .ok_or_else(|| "unexpected end of input".to_string())
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_ws();
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Ok(Value::Str(self.string()?)),
            b't' | b'f' => self.boolean(),
            b'n' => self.null(),
            b'-' | b'0'..=b'9' => self.number(),
            other => Err(format!(
                "unexpected byte {:?} at {}",
                other as char, self.pos
            )),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.pos += 1; // consume '{'
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek()? == b'}' {
            self.pos += 1;
            return Ok(Value::Obj(out));
        }
        loop {
            self.skip_ws();
            if self.peek()? != b'"' {
                return Err(format!("expected object key at {}", self.pos));
            }
            let key = self.string()?;
            self.skip_ws();
            if self.peek()? != b':' {
                return Err(format!("expected ':' at {}", self.pos));
            }
            self.pos += 1;
            let val = self.value()?;
            out.push((key, val));
            self.skip_ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b'}' => {
                    self.pos += 1;
                    return Ok(Value::Obj(out));
                }
                _ => return Err(format!("expected ',' or '}}' at {}", self.pos)),
            }
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.pos += 1; // consume '['
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek()? == b']' {
            self.pos += 1;
            return Ok(Value::Arr(out));
        }
        loop {
            let val = self.value()?;
            out.push(val);
            self.skip_ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b']' => {
                    self.pos += 1;
                    return Ok(Value::Arr(out));
                }
                _ => return Err(format!("expected ',' or ']' at {}", self.pos)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.pos += 1; // consume opening quote
        let mut out = String::new();
        loop {
            let b = self.peek()?;
            self.pos += 1;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self.peek()?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b't' => out.push('\t'),
                        b'r' => out.push('\r'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'u' => {
                            let cp = self.hex4()?;
                            // No surrogate-pair handling: contracts use BMP text.
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                        }
                        other => return Err(format!("bad escape \\{}", other as char)),
                    }
                }
                // Raw byte: rebuild UTF-8 by collecting the continuation bytes.
                _ => {
                    let start = self.pos - 1;
                    let len = utf8_len(b);
                    let end = start + len;
                    if end > self.bytes.len() {
                        return Err("truncated UTF-8 in string".to_string());
                    }
                    let s = std::str::from_utf8(&self.bytes[start..end])
                        .map_err(|_| "invalid UTF-8 in string".to_string())?;
                    out.push_str(s);
                    self.pos = end;
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        if self.pos + 4 > self.bytes.len() {
            return Err("truncated \\u escape".to_string());
        }
        let mut cp = 0u32;
        for _ in 0..4 {
            let c = self.bytes[self.pos] as char;
            let d = c.to_digit(16).ok_or_else(|| "bad \\u hex".to_string())?;
            cp = cp * 16 + d;
            self.pos += 1;
        }
        Ok(cp)
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E' => self.pos += 1,
                _ => break,
            }
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "bad number bytes".to_string())?;
        s.parse::<f64>()
            .map(Value::Num)
            .map_err(|_| format!("invalid number {s:?}"))
    }

    fn boolean(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(Value::Bool(true))
        } else if self.bytes[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(Value::Bool(false))
        } else {
            Err(format!("invalid literal at {}", self.pos))
        }
    }

    fn null(&mut self) -> Result<Value, String> {
        if self.bytes[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(Value::Null)
        } else {
            Err(format!("invalid literal at {}", self.pos))
        }
    }
}

/// Length in bytes of a UTF-8 sequence given its leading byte.
fn utf8_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_contract_shape() {
        let v = parse(
            r#"{ "intent": "decide", "facts": [ {"key": "clause:0", "value": "1 2"} ],
                "candidates": [], "state": [] }"#,
        )
        .unwrap();
        assert_eq!(v.get("intent").and_then(Value::as_str), Some("decide"));
        let facts = v.get("facts").and_then(Value::as_array).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(
            facts[0].get("key").and_then(Value::as_str),
            Some("clause:0")
        );
        assert_eq!(facts[0].get("value").and_then(Value::as_str), Some("1 2"));
    }

    #[test]
    fn decodes_escapes_and_unicode() {
        let v = parse(r#"{"s": "a\t\"b\"é"}"#).unwrap();
        assert_eq!(v.get("s").and_then(Value::as_str), Some("a\t\"b\"é"));
    }

    #[test]
    fn rejects_trailing_garbage() {
        assert!(parse("{} extra").is_err());
        assert!(parse("[1, 2,]").is_err());
    }

    #[test]
    fn parses_numbers_and_bools() {
        let v = parse(r#"{"n": -3.5e2, "b": true, "z": null}"#).unwrap();
        assert_eq!(v.get("n"), Some(&Value::Num(-350.0)));
        assert_eq!(v.get("b"), Some(&Value::Bool(true)));
        assert_eq!(v.get("z"), Some(&Value::Null));
    }
}
