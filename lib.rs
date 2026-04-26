//! # A set of functions used for performing common tasks of a web application
//!
//! # Examples
//! Consider web **hello, world** as:
//!
//! ```rust
//! use simweb::WebPage;
//!
//! struct Hello;
//!
//! fn main() {
//!    Hello{}.show()
//! }
//! 
//! impl WebPage for Hello {
//!    fn main_load(&self) -> Result<String, String> {
//!        Ok(r#"<!doctype html>
//! <html><body>Hello, the web world</body></html>"#.to_string ())
//!     }
//! }
//!
//! ```
mod data;
mod mpart;
mod simweb;
mod template;
mod util;

pub use data::{
    WebData, adjust_separator, as_web_path, base64_encode_with_padding, 
    http_format_time, parse_http_timestamp, sanitize_web_path,
};
pub use mpart::MPart;
pub use simweb::WebPage;
pub use template::{Selectable, interpolate};
pub use util::list_files;

use std::{borrow::Cow, error::Error, fmt, time::SystemTime, env,};

const VERSION: &str = env!("VERSION");

/// A struture to hold 5xx http errors
pub struct FiveXXError {}

/// The stucture holds an error details
#[derive(Debug)]
pub struct WebError {
    pub reason: String,
    pub cause: Option<Box<dyn Error>>,
}

impl Error for WebError {}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {}, caused by: {:?}", self.reason, self.cause)
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
        Err(Box::new(WebError {
            reason: "impl error".to_string(),
            cause: None,
        }))
    }
}

/// Returns a current version of the crate
///
/// It helps to report problems
pub fn get_version() -> &'static str {
    VERSION
}

/// Creates a cookie header String
///
/// # Parameters
/// * name - a name of the cookie
/// * value - a value of the cookie
/// * expiration - an optional expiration date as `SystemTime`, it will be a session cookie, when `None`
///
/// # Examples
/// ```
/// let (set_op, val) = new_cookie_header("age", "23", None);
/// ```
pub fn new_cookie_header(
    name: &str,
    value: &str,
    expiration: Option<SystemTime>,
) -> (String, String) {
    if let Some(time) = expiration {
        (
            "Set-Cookie".to_string(),
            format! {"{name}={value}; Expires={}", data::http_format_time(time)},
        )
    } else {
        ("Set-Cookie".to_string(), format! {"{name}={value}"})
    }
}

/// HTML encode a given String
///
/// encodes specific to HTML characters preventing them to be interpreted as HTML elements
///
/// # Examples
///
/// ```
/// let encoded = html_encode("<tag>");
/// assert_eq!("&lt;tag&gt;", encoded);
/// ```
///
pub fn html_encode(orig: &impl AsRef<str>) -> String {
    // TODO consider using Cow
    let s = orig.as_ref();
    let chars = s.chars();
    let mut res = String::with_capacity(s.len());
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

/// Encodes a `&str` to use in JSON values
///
/// # Return
/// A str itself, or a new `String` when encoding happens
///
/// # Examples
/// ```
/// assert_eq!(json_encode(r#"This is
/// Rust"#), "This is\nRust");
/// ```
pub fn json_encode(orig: &str) -> Cow<'_, str> {
    let (extra, offs) = escaped_len(orig);
    if extra > 0 {
        let mut res = String::with_capacity(extra + orig.len());
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

fn escaped_len(s: &str) -> (usize, usize) {
    let mut chars = s.char_indices();
    let mut res = 0_usize;
    let mut offs = 0;
    for (i, c) in chars.by_ref() {
        let esc_len = escape_char(c);
        if esc_len > 0 {
            res += esc_len;
            offs = i;
            break;
        }
    }
    for (_i, c) in chars {
        //res += escape_char(c);
        let esc_len = escape_char(c);
        if esc_len > 0 {
            res += esc_len
        }
    }
    (res, offs)
}

#[inline]
fn escape_char(c: char) -> usize {
    match c {
        '"' | '\n' | '\r' | '\t' | '\\' => 1,
        '\u{0000}'..'\u{1f}' => 5,
        _ => 0,
    }
}

/// Returns the path info.
///
/// If there is no path info, then an empty `String` is returned.
/// A path info can't be as an empty `String`.
pub fn path_info() -> String {
    env::var("PATH_INFO").unwrap_or_default()
}

/// Decodes URL component.
///
/// If a decoding impossible, the it returns `None`.
///
/// A new string is always created regardless if an actual decoding happened.
pub fn url_comp_decode(comp: &str) -> Option<String> {
    let mut res = Vec::with_capacity(256);

    let mut chars = comp.chars();
    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let d1 = chars.next()?.to_digit(16)?;
                let d2 = chars.next()?.to_digit(16)?;
                res.push(((d1 << 4) + d2) as u8)
            }
            '+' => res.push(b' '),
            _ => res.push(if c.is_ascii() { c as u8 } else { return None }),
        }
    }
    String::from_utf8(res).ok()
}

/// It's encoding as URL component encode
pub fn url_encode(orig: impl AsRef<str>) -> String {
    let s = orig.as_ref();
    let chars = s.chars();
    let mut res = String::with_capacity(s.len());
    let mut b = [0; 4];
    for c in chars {
        if (c as u32) < 256
            && matches!(c as u8, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' |  b'-' | b'.' | b'_' | b'~')
        {
            res.push(c)
        } else {
            b.fill(0);
            c.encode_utf8(&mut b);
            let mut b_it = b.iter();
            res.push_str(&format! {"%{:02x}", b_it.next().unwrap()});
            for b in b_it {
                if *b == 0 {
                    break; //continue
                }
                res.push_str(&format! {"%{:02x}", b})
            }
        }
    }
    res
}

/// Encloses a given String in the left and right brackets.
///
/// # Examples
/// ```
///  println!("{}", enclose("html", "<", ">"));
///  println!("{}", enclose("Hello, Web", "\"", "\""));
/// ```
pub fn enclose(s: &str, left: &str, right: &str) -> String {
    let mut res = String::with_capacity(s.len() + left.len() + right.len());
    res.push_str(left);
    res.push_str(s);
    res.push_str(right);
    res
}

/// Converts an array of bytes to hex value suitable for printing
pub fn to_hex(line: &[u8]) -> String {
    let mut s = String::with_capacity(2 * line.len());
    use std::fmt::Write as FmtWrite; // renaming import to avoid collision
    for b in line {
        write!(s, "{:02x}", b).unwrap();
    }
    s
}
