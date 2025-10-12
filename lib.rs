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
    if let Some(esc_len) = escaped_len(orig) {
        let mut res = String::with_capacity(esc_len);
        for c in orig.chars() {
            if let Some(c_esc) = escape_char(c) {
                res.push_str(c_esc);
            } else {
                // Note: There is probably some missed optimization potential here
                // because `c` was just UTF-8-decoded and now we re-encode it.
                res.push(c);
            }
        }
        Cow::Owned(res)
    } else {
        Cow::Borrowed(orig)
    }
}

fn escaped_len(mut s: &str) -> Option<usize> {
    let mut len = 0;
    let mut escaped = false;
    while let Some((before, esc, after)) = first_escaped_char(s) {
        len += before.len() + esc.len();
        escaped = true;
        s = after;
    }
    if escaped {
        len += s.len();
        Some(len)
    } else {
        None
    }
}

fn first_escaped_char<'a>(s: &'a str) -> Option<(&'a str, &'static str, &'a str)> {
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        if let Some(c_esc) = escape_char(c) {
            return Some((&s[..i], c_esc, &s[chars.offset()..]));
        }
    }
    None
}

fn escape_char(c: char) -> Option<&'static str> {
    match c {
        '"' => Some("\\\""),
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        '\\' => Some("\\\\"),
        _ => None,
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
            for i in 1..b.len() {
                if b[i]==0 {
                    //continue
                    break 
                }
                res.push_str(&format!{"%{:02x}", b[i]})
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