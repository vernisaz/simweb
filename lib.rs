mod simweb;
mod template;
mod data;
mod util;

pub use simweb::WebPage;
pub use data::WebData;
pub use data::{http_format_time, has_root, as_web_path, adjust_separator};
pub use util::list_files;

pub struct FiveXXError {}

use std::fmt;
use std::time::SystemTime;

impl fmt::Debug for FiveXXError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.show();
        Ok(())
    }
}

impl WebPage for FiveXXError {
    fn main_load(&self) -> Result<String, String> {
        Err("Rust impl error".to_string())
    }
    
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

pub fn json_encode(orig: &impl AsRef<str>) -> String {
    let chars = orig.as_ref().chars();
    let mut res = String::from("");
    for c in chars {
        match c {
            '"' => res.push_str("\\\""),
            '\n' => res.push_str("\x5Cn"),
            '\r' => res.push_str("\x5cr"),
            '\t' => res.push_str("\x5ct"),
            '\\' => res.push_str("\x5c\x5c"),
            _ => res.push(c),
        }
    }
    res
}

/// it's an analog of URL component encode
pub fn url_encode(orig: &impl AsRef<str>) -> String {
    let chars = orig.as_ref().chars();
    let mut res = String::from("");
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