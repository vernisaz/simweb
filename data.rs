use crate::WebError;
use simtime::{get_datetime, seconds_from_epoch};
#[cfg(any(unix, target_os = "redox"))]
use std::path::MAIN_SEPARATOR_STR;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::{MAIN_SEPARATOR, Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug)]
pub struct WebData {
    params: HashMap<String, String>, // &str or (String, Option<Vec<String>>)
    params_dup: HashMap<String, Vec<String>>,
    cookies: HashMap<String, String>,
    pub query: Option<String>,
}

pub const HTTP_DAYS_OF_WEEK: &[&str] = &["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];

pub const HTTP_MONTH: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
    "Dec", // begins with epoch start day
];

impl Default for WebData {
    fn default() -> Self {
        Self::new()
    }
}

impl WebData {
    pub fn new() -> Self {
        let mut res = WebData {
            params: HashMap::new(),
            params_dup: HashMap::new(),
            cookies: HashMap::new(),
            query: None,
        };
        if let Ok(query) = env::var("QUERY_STRING") {
            let parts = query.split("&");
            for part in parts {
                if let Some((key, val)) = part.split_once("=") {
                    let key = res.url_comp_decode(key);
                    if let Some(prev) = res.params.insert(key.clone(), res.url_comp_decode(val)) {
                        let others = res.params_dup.get_mut(&key);
                        match others {
                            None => {
                                let params = vec![prev];
                                res.params_dup.insert(key, params);
                            }
                            Some(others) => others.push(prev),
                        }
                    };
                }
            }
        }
        if let Ok(header_cookies) = env::var("HTTP_COOKIE") {
            let parts = header_cookies.split(";");
            for part in parts {
                if let Some(keyval) = part.split_once('=') {
                    res.cookies
                        .insert(keyval.0.trim().to_string(), keyval.1.to_string());
                }
            }
        } else {
            // eprintln!{"No cookie header"}
        }

        if let Ok(method) = env::var("REQUEST_METHOD")
            && method == "POST"
        {
            let mut length = 0;
            if let Ok(content_length) = env::var("CONTENT_LENGTH")
                && let Ok(content_length) = content_length.parse::<u64>()
            {
                length = content_length
            }
            let mut user_input = String::new();
            let stdin = io::stdin();
            if let Ok(content_type) = env::var("CONTENT_TYPE") {
                match content_type.as_str() {
                    "application/x-www-form-urlencoded" => {
                        if let Ok(_ok) = stdin.read_line(&mut user_input) {
                            let parts = user_input.split("&");
                            for part in parts {
                                if let Some((key, val)) = part.split_once("=") {
                                    let key = res.url_comp_decode(key);
                                    if let Some(prev) =
                                        res.params.insert(key.clone(), res.url_comp_decode(val))
                                    {
                                        let others = res.params_dup.get_mut(&key);
                                        match others {
                                            None => {
                                                let params = vec![prev];
                                                res.params_dup.insert(key, params);
                                            }
                                            Some(others) => others.push(prev),
                                        }
                                    };
                                }
                            }
                        }
                        // sink reminded if any
                    }
                    _ if content_type.starts_with("multipart/form-data;") => {
                        match parse_multipart(
                            &content_type,
                            stdin,
                            length as usize,
                            &mut res.params,
                            &mut res.params_dup,
                        ) {
                            Ok(()) => (),
                            Err(err) => {
                                eprintln! {"error: parse multi part failed = {err}"}
                            }
                        }
                        // sink reminded if any
                    }
                    _ => (), // sink reminded if any
                }
            } else {
                // read by end
            }
        }
        res
    }

    pub fn param(&self, key: impl AsRef<str>) -> Option<String> {
        self.params.get(key.as_ref()).cloned() // probably better to return as Option<&String> without using clone
    }

    pub fn params(&self, key: impl AsRef<str>) -> Option<Vec<String>> {
        let key = key.as_ref();
        match self.params.get(key) {
            None => None,
            Some(val) => {
                let mut res = vec![val.clone()];
                match self.params_dup.get(key) {
                    None => Some(res),
                    Some(vec) => {
                        vec.iter().for_each(|el| res.push(el.clone()));
                        Some(res)
                    }
                }
            }
        }
    }

    pub fn cookie(&self, key: impl AsRef<str>) -> Option<String> {
        self.cookies.get(key.as_ref()).cloned() // probably better to return as Option<&String> without using clone
    }

    pub fn path_info(&self) -> String {
        if let Ok(pi) = env::var("PATH_INFO") {
            pi.to_string()
        } else {
            // there is no clash since path info is never an empty string
            "".to_string()
        }
    }

    pub fn url_comp_decode(&self, comp: &str) -> String {
        let mut res = Vec::with_capacity(256);

        let mut chars = comp.chars();
        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    if let Some(c1) = chars.next() {
                        if let Some(d1) = c1.to_digit(16) {
                            if let Some(c2) = chars.next() {
                                if let Some(d2) = c2.to_digit(16) {
                                    res.push(((d1 << 4) + d2) as u8)
                                } else {
                                    res.push(if c1.is_ascii() { c as u8 } else { b'?' });
                                    res.push(if c2.is_ascii() { c as u8 } else { b'?' })
                                }
                            }
                        } else {
                            res.push(if c1.is_ascii() { c as u8 } else { b'?' })
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

use crate::mpart::MPart;
// TODO make a method with self
fn parse_multipart(
    content_type: &str,
    mut stdin: io::Stdin,
    length: usize,
    res: &mut HashMap<String, String>,
    res_dup: &mut HashMap<String, Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let Some((_, boundary)) = content_type.split_once("; boundary=") else {
        return Err(Box::new(WebError {
            reason: "No boundary".to_string(),
            cause: None,
        }));
    };
    let parts = MPart::from(&mut stdin, boundary.as_bytes());
    let mut consumed = 0_usize;

    for part in parts {
        // eprintln!{"part {:?} / {:?} / {}",part.content_type, part.content_filename, &part.total_read_ammount}

        let insert = |val| {
            if let Some(prev) = res.insert(
                part.content_name.clone(),
                val, //String::from_utf8(part.content.clone()).unwrap()
            ) {
                let others = res_dup.get_mut(&part.content_name);
                match others {
                    None => {
                        let params = vec![prev];
                        res_dup.insert(part.content_name, params);
                    }
                    Some(others) => others.push(prev),
                }
            };
        };

        consumed = part.total_read_ammount;
        match part.content_type {
            None => insert(String::from_utf8(part.content).unwrap()),
            // TODO apply any encoding if specified
            Some(content_type)
                if part.content_filename.is_none() && content_type.starts_with("text/") =>
            {
                insert(iso_8859_1_to_string(&part.content))
            }
            _ => match part.content_filename {
                Some(content_filename) => {
                    let atdir = match env::var("ATTACH_DIR") {
                        Ok(dir) => dir,
                        Err(_) => env::current_dir()?.into_os_string().into_string().unwrap(),
                    };
                    let mut file_name = PathBuf::from(atdir);
                    file_name.push(content_filename);
                    match write_to_file(part.content, file_name.to_str().unwrap()) {
                        Ok(_) => {
                            println!("File written successfully!");
                            insert(file_name.to_str().unwrap().to_string());
                        }
                        Err(e) => eprintln!("Failed to write file: {}", e),
                    };
                }
                _ => eprintln! {"can't save field, since no file name"},
            },
        };
    }
    if length != consumed {
        if length > consumed {
            let mut buffer = vec![0_u8; length - consumed];
            stdin.read_exact(&mut buffer[..]).unwrap();
        }
        Err(Box::new(WebError {
            reason: "Size mismatch".to_string(),
            cause: None,
        }))
    } else {
        Ok(())
    }
}

fn write_to_file(data: Vec<u8>, file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(Path::new(file_path))?;
    file.write_all(&data)?;
    Ok(())
}

fn iso_8859_1_to_string(s: &[u8]) -> String {
    s.iter().map(|&c| c as char).collect()
}

pub fn http_format_time(time: SystemTime) -> String {
    let dur = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let (y, m, d, h, min, s, w) = get_datetime(1970, dur.as_secs());
    format!(
        "{}, {d:0>2} {} {y:0>2} {h:0>2}:{min:0>2}:{s:0>2} GMT",
        HTTP_DAYS_OF_WEEK[w as usize],
        HTTP_MONTH[(m - 1) as usize]
    )
}

pub fn parse_http_timestamp(str: &str) -> Result<u64, &str> {
    let (_, date) = str.split_once(", ").ok_or("invalid timestamp string")?;
    let parts: Vec<_> = date.split_ascii_whitespace().collect();
    let [day, month, year, time, _tz] = parts.as_slice() else {
        return Err("invalid timestamp parts");
    };
    let day = day.parse::<u32>().map_err(|_| "day isn't a number")?;
    let month: u32 = match *month {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return Err("month not valid string"),
    };
    let year = year.parse::<u32>().map_err(|_| "year isn't a number")?;
    let [h, m, s] = *time.splitn(3, ':').collect::<Vec<_>>() else {
        todo!()
    };
    let h = h.parse::<u32>().map_err(|_| "hour isn't a number")?;
    let m = m.parse::<u32>().map_err(|_| "minute isn't a number")?;
    let s = s.parse::<u32>().map_err(|_| "second isn't a number")?;
    seconds_from_epoch(1970, year, month, day, h, m, s)
}

pub fn adjust_separator(mut path: String) -> String {
    let foreign_slash = if MAIN_SEPARATOR == '\\' { '/' } else { '\\' };
    let vec = unsafe { path.as_mut_vec() };
    for el in vec {
        if *el == foreign_slash as u8 {
            *el = MAIN_SEPARATOR as u8;
        }
    }

    path
}

pub fn sanitize_web_path(path: String) -> Result<String, WebError> {
    // the string considered as URL decoded
    for part in path.split("/") {
        if part == ".." {
            return Err(WebError {
                reason: "The path contains prohibited .. element".to_string(),
                cause: None,
            });
        }
    }
    Ok(path)
}

pub fn as_web_path(path: &mut str) -> &str {
    unsafe {
        let path_vec: &mut [u8] = path.as_bytes_mut();
        for el in path_vec {
            if *el == b'\\' {
                *el = b'/';
            }
        }
    }
    path
}

const BASE64: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode_with_padding(input: &[u8]) -> String {
    let mut remain = 0_u8;
    let mut remain_len = 0;
    let base64 = BASE64.as_bytes();
    let mut res = String::new();
    res.reserve(input.len() + (input.len() + 5) / 3);
    for b in input {
        match remain_len {
            0 => {
                remain = b & 3;
                let i = b >> 2;
                remain_len = 2;
                res.push(base64[i as usize] as char);
            }
            2 => {
                let i = remain << 4 | (b >> 4);
                res.push(base64[i as usize] as char);
                remain = b & 15;
                remain_len = 4;
            }
            4 => {
                let i = remain << 2 | ((b >> 6) & 3);
                res.push(base64[i as usize] as char);
                remain = 0;
                remain_len = 0;
                res.push(base64[(b & 63) as usize] as char);
            }
            _ => (),
        }
    }
    // padding
    match remain_len {
        2 => {
            res.push(base64[(remain << 4) as usize] as char);
            res.push_str("==")
        }
        4 => {
            res.push(base64[(remain << 2) as usize] as char);
            res.push('=')
        }
        _ => (),
    }
    res
}

#[cfg(target_os = "windows")]
pub fn has_root(path: impl AsRef<str>) -> bool {
    let path = path.as_ref().as_bytes();
    path.len() > 3 && path[1] == b':' && path[2] == b'\\'
        || path.len() > 0 && path[0] == MAIN_SEPARATOR as _
}

#[cfg(any(unix, target_os = "redox"))]
#[inline]
pub fn has_root(path: impl AsRef<str>) -> bool {
    path.as_ref().starts_with(MAIN_SEPARATOR_STR)
}
