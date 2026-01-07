mod simweb;
mod template;
mod data;
mod util;
mod mpart;

pub use simweb::WebPage;
pub use data::{http_format_time, parse_http_timestamp, has_root, as_web_path, adjust_separator,
    base64_encode_with_padding, WebData, sanitize_web_path};
pub use util::list_files;
pub use mpart::{MPart, };
pub use template::{Selectable,interpolate};

use std::{fmt,
    time::SystemTime, error::Error, borrow::Cow,
    };
    
const VERSION: &str = env!("VERSION");

pub struct FiveXXError {}

#[derive(Debug)]
pub struct WebError {
    pub reason: String,
    pub cause: Option<Box<dyn std::error::Error>>,
}

impl Error for WebError {
    
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "It's because {}, optionally: {:?}", self.reason, self.cause)
    }
}
 
impl fmt::Debug for FiveXXError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.show();
        Ok(())
    }
}

impl WebPage for FiveXXError {
    fn main_load(&self) -> Result<String, Box<dyn std::error::Error>> {
        Err(Box::new(WebError{reason:"Rust impl error".to_string(), cause:None}))
    }
    
}

pub fn get_version() -> &'static str {
    VERSION
}

pub fn new_cookie_header(name: &String, value: &String, exparation: Option<SystemTime>) -> (String, String) {
    if let Some(time) = exparation {
        ("Set-Cookie".to_string(), format!{"{name}={value}; Expires={}", data::http_format_time(time)})
    } else {
        ("Set-Cookie".to_string(), format!{"{name}={value}"})
    }
}

pub fn html_encode(orig: &impl AsRef<str>) -> String {
    let chars = orig.as_ref(). chars();
    let mut res = String::from("");
    for c in chars {
        match c {
            '<' => res.push_str("&lt;"),
            '>' => res.push_str("&gt;"),
            '"' => res.push_str("&quot;"),
            '\'' => res.push_str("&#39;"),
            '&' => res.push_str("&amp;"),
            _ => res.push(c),
        }
    }
    res
}

pub fn json_encode(orig: &str) -> Cow<'_, str> {
    let (extra, offs)  = escaped_len(orig);
    if extra > 0 {
        let mut res = String::with_capacity(extra+orig.len());
        if offs > 0 {
            res.push_str(&orig[..offs])
        }
        for c in orig[offs..].chars() {
            match c {
            '"' => res.push_str("\\\""),
            '\n' => res.push_str("\\n"),
            '\r' => res.push_str("\\r"),
            '\t' => res.push_str("\\t"),
            '\\' => res.push_str("\\\\"),
            '\u{0000}'..'\u{1f}' => res.push_str(&format!("\\u00{:02x}", c as u8)), 
            _ => res.push(c),
            }
        }
        Cow::Owned(res)
    } else {
        Cow::Borrowed(orig)
    }
}

fn escaped_len(s: &str) -> (usize,usize) {
    let mut chars = s.char_indices();
    let mut res = 0_usize;
    let mut offs = 0;
    for (i, c) in chars.by_ref() {
        let esc_len = escape_char(c);
        if esc_len > 0 { res += esc_len; offs = i; break }
    }
    for (_i, c) in chars {
        //res += escape_char(c);
        let esc_len = escape_char(c);
        if esc_len > 0 {
            res += esc_len
        }
    }
    (res,offs)
}

#[inline]
fn escape_char(c: char) -> usize {
    match c {
        '"' | '\n' | '\r' | '\t' | '\\' => 1,
        '\u{0000}'..'\u{1f}' => 5, 
        _ => 0,
    }
}

/// it's encoding as URL component encode
pub fn url_encode(orig: &impl AsRef<str>) -> String {
    let chars = orig.as_ref().chars();
    let mut res = String::new();
    let mut b = [0; 4];
    for c in chars {
        if (c as u32) < 256 && matches!(c as u8, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~') {
            res.push(c)
        } else {
            b.fill(0);
            c.encode_utf8(&mut b);
            res.push_str(&format!{"%{:02x}", b[0]});
            for b in b.iter().skip(1) {
                if *b==0 {
                    break //continue
                }
                res.push_str(&format!{"%{:02x}", b})
            }
        }
    }
    res
}

pub fn to_hex(line: &[u8]) -> String {
    let mut s = String::new();
    use std::fmt::Write as FmtWrite; // renaming import to avoid collision
    for b in line { write!(s, "{:02x}", b).unwrap(); }
    s
}