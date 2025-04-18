use std::{collections::HashMap,
    io::{self,Read,ErrorKind},
    time::SystemTime,
    path::{MAIN_SEPARATOR}};
#[cfg(any(unix, target_os = "redox"))]
use std::path::{MAIN_SEPARATOR,MAIN_SEPARATOR_STR};
#[cfg(target_os = "windows")]

use simtime::get_datetime;

#[derive(Debug)]
pub struct WebData {
    params: HashMap<String, String>, // &str
    cookies: HashMap<String, String>,
    pub query: Option<String>,
}

pub const HTTP_DAYS_OF_WEEK: &[&str] = &[
"Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed",];

pub const HTTP_MONTH: &[&str] = &[
"Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec", 
];

impl WebData {
    pub fn new() -> Self {
        let mut res = WebData {
            params: HashMap::new(),
            cookies: HashMap::new(),
            query: None,
        };
        if let std::result::Result::Ok(query) = std::env::var(String::from("QUERY_STRING")) {
            let parts = query.split("&");
            for part in parts {
                if let Some(keyval) = part.split_once("=") {
                    res.params.insert(
                        res.url_comp_decode(&keyval.0.to_string()),
                        res.url_comp_decode(&keyval.1.to_string())
                    );
                }
            }
        }
        if let std::result::Result::Ok(header_cookies) = std::env::var(String::from("HTTP_COOKIE")) {
            let parts = header_cookies.split(";");
            for part in parts {
                if let Some(keyval) = part.split_once('=') {
                    res.cookies.insert(
                        keyval.0.trim().to_string(),
                        keyval.1.to_string(),
                    );
                }
            }
        } else {
           // eprintln!{"No cookie header"}
        }

        if let std::result::Result::Ok(method) = std::env::var(String::from("REQUEST_METHOD")) {
            if method == "POST" 
            {
                let mut length = 0;
                if let Ok(content_length) = std::env::var(String::from("CONTENT_LENGTH")) {
                    if let Ok(content_length) = content_length.parse::<u64>() {
                        length = content_length
                    }
                }
                let mut user_input = String::new();
                let stdin = io::stdin();
                if let Ok(content_type) = std::env::var(String::from("CONTENT_TYPE")) {
                    match  content_type.as_str() {
                        "application/x-www-form-urlencoded" => {
                            if let Ok(_ok) = stdin.read_line(&mut user_input) {
                                let parts = user_input.split("&");
                                for part in parts {
                                    if let Some(keyval) = part.split_once('=') {
                                        res.params.insert(
                                            res.url_comp_decode(&keyval.0.to_string()),
                                            res.url_comp_decode(&keyval.1.to_string()),
                                        );
                                    }
                                }
                            }
                            // sink reminded if any
                        }
                        _ if content_type.starts_with("multipart/form-data;") => {
                            // TODO fimnd a soltion to get data in binary for attahments or something
                            parse_multipart(&content_type, stdin, length as usize, &mut res.params).unwrap()
                            // sink reminded if any
                        }
                        _ => () // sink reminded if any
                    }
                } else {
                    // read by end
                }
            }
        }
        res
    }

    pub fn param(&self, key: impl AsRef<str>) -> Option<String> {
        self.params.get(key.as_ref()).cloned() // probably better to return as Option<&String> without using clone
    }
    
    pub fn cookie(&self, key: impl AsRef<str>) -> Option<String> {
        self.cookies.get(key.as_ref()).cloned() // probably better to return as Option<&String> without using clone
    }

    pub fn path_info(&self) -> String {
        if let std::result::Result::Ok(pi) = std::env::var(String::from("PATH_INFO")) {
            pi.to_string()
        } else {
        // since path info is never an empty string
            "".to_string()
        }
    }

    pub fn url_comp_decode(&self, comp: &String) -> String {
        let mut res = Vec::with_capacity(256);

        let mut chars = comp.chars();
        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    if let Some(c1) = chars.next() {
                        let d1 = c1.to_digit(16).unwrap();
                        if let Some(c2) = chars.next() {
                            let d2 = c2.to_digit(16).unwrap();
                            res.push(((d1 << 4) + d2) as u8)
                        }
                    }
                }
                '+' => res.push(b' '),
                _ => res.push(if c.is_ascii() { c as u8 } else { b'?' }),
            }
        }
        String::from_utf8_lossy(&res).to_string()
    }

}

fn parse_multipart(content_type: &String, mut stdin: io::Stdin, length: usize, res: &mut HashMap<String,String>) -> io::Result<()> {
    let Some(keyval) = content_type.split_once("; boundary=") else {
        let mut buffer = Vec::new();
        // read the whole file
        stdin.read_to_end(&mut buffer)?;
        return if length != buffer.len() {
            Err(io::Error::new(ErrorKind::Other, "Size mismatch"))
        } else {
        
             Ok(())
        }
    };
    Ok(())
}

pub fn http_format_time(time: SystemTime) -> String {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let (y, m, d, h, min, s, w) = get_datetime(1970, dur.as_secs());
    format!("{}, {d:0>2} {} {y:0>2} {h:0>2}:{min:0>2}:{s:0>2} GMT",
         HTTP_DAYS_OF_WEEK[w as usize], HTTP_MONTH[(m-1) as usize])
}

pub fn adjust_separator(mut path: String) -> String {
    let foreign_slash = if MAIN_SEPARATOR == '\\' { '/' } else { '\\' };
    let vec = unsafe {path.as_mut_vec()};
    for c in 0..vec.len() {
        if vec[c] == foreign_slash as u8 { vec[c] = MAIN_SEPARATOR as u8;}
    }

    path
}

pub fn as_web_path(mut path: String ) -> String {
    unsafe {
        let path_vec: &mut [u8]= path.as_bytes_mut();
    
        for c in 0..path_vec.len() {
            if path_vec[c] == b'\\' { path_vec[c] = b'/';}
        }
    }
    path
}

#[cfg(target_os = "windows")]
pub fn has_root(path:  impl AsRef<str>) -> bool {
    let path = path.as_ref().as_bytes();
    path.len() > 3 && path[1] == b':' && path[2] == b'\\' || path.len() > 0 && path[0] == MAIN_SEPARATOR as _
}

#[cfg(any(unix, target_os = "redox"))]
#[inline]
pub fn has_root(path:  impl AsRef<str>) -> bool {
    path.as_ref().starts_with(MAIN_SEPARATOR_STR)
}