extern crate simweb;
extern crate simtime;
use std::fs::read_to_string;
use std::time::SystemTime;
use std::collections::HashMap;
#[cfg(any(unix, target_os = "redox"))]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::ops::ControlFlow;
use ControlFlow::Continue;

use simweb::FiveXXError;
use simweb::WebPage;

struct Page {
    dir: String
}

fn main()  -> Result<(), FiveXXError> {
   Ok(Page { dir:
     if let Ok(current_path) = std::env::var(String::from("PATH_TRANSLATED")) {
        let path = Path::new(&current_path);
        format!{r#"<!doctype html>
<html>
  <head>
    <title>Test content from {current_path}</title>
    <style>
    a {{
      color: #0033cc;
    }}
    @media (prefers-color-scheme: dark) {{
      body {{
        color: #eee;
        background: #121212;
      }}
      a {{
       color: #99ccff;
      }}
    }}
   </style>
  </head>
  <body>
    <h2>{current_path}</h2>
    <p style="color:#6666cc">${{timestamp}}</p>
    {}
  </body>
</html>"#, if path.is_file() {
                format!{"<pre>{}</pre>", read_to_string(&current_path).map_err(|_| FiveXXError{})?} 
            } else if path.is_dir() {
                match path.read_dir() {
                    Ok(dir) => {
                        let mut dir_cont = String::from("");
                        let web_path = std::env::var(String::from("PATH_INFO")).unwrap();
                        if web_path .len() > 1 {
                            dir_cont.push_str("<a href=\"..\">..</a><br>")
                        }
                        // URL encoded
                        let encoded = web_path.split_terminator('/').try_fold(String::from(""), |res, el| 
                            if el.is_empty() {
                                ControlFlow::<String, String>::Continue(res)
                            } else {
                                ControlFlow::Continue(res + "/" + &simweb::url_encode(&el))
                            });
                        let encoded = match encoded {
                            Continue(res) => res,
                            _ => "".to_string()
                        };
                        dir_cont.push_str(&format!{"<!-- {web_path} -> {encoded} --><table>"});
                        let local = simweb::WebData::new().param("localtime").is_some();
                        for entry in dir {
                            if let Ok(entry) = entry {
                                let file_name = entry.file_name().to_str().unwrap().to_owned();
                                let slash = if entry.file_type().map_err(|_| FiveXXError{})?.is_dir() { "/" } else { "" };
                                dir_cont.push_str(&format!("<tr><td id=\"rr0\"><a  href=\"./{1}{slash}\">{0}</a></td>", simweb::html_encode(&file_name),
                                      simweb::url_encode(&file_name)));
                                let metadata = entry.metadata().map_err(|_| FiveXXError{})?;
                                #[cfg(target_os = "windows")]
                                let mode = if metadata.permissions().readonly() {0o444}else{0o777};
                                #[cfg(target_os = "linux")]
                                let mode =  metadata.permissions().mode();
                                dir_cont.push_str(&format!{r#" <td>{}</td><td style=\"text-align: end; padding-right: 1em;\">{}</td>
                                <td style=\"text-align: center; padding-right: 1em;\">{:0>16}</td><td>{:0>3o}</td></tr>"#, 
                                if metadata.is_dir() { simweb::html_encode(&"<DIR>")} else  {"file".to_string()},
                                metadata.len(), 
                                format_time(metadata.modified().map_err(|_| FiveXXError{})?, local), mode})
                            }
                        }
                        dir_cont.push_str("</table>");
                        dir_cont
                    }
                    Err(_) => format!{"Can't read {current_path}"}.to_string()
                }
            } else {
                format!{"{current_path} doesn't exist"}.to_string()
            }
        }
     } else {
        String::from(r#"<!doctype html>
<html>
  <head>
    <title>Test content from</title>
  </head>
  <body>
    <p>No ${name}</p>
  </body>
</html>"#)
    }
    }.show()) 
}

impl simweb::WebPage for Page {
    fn main_load(&self) -> Result<String, String> {
        Ok(self.dir.clone())
    }
    
    fn apply_specific(&self, page_map: &mut HashMap<String, String>) {
        page_map.insert(String::from("timestamp"),
        simweb::http_format_time(SystemTime::now()));
    }

}


fn format_time(time: SystemTime, local: bool) -> String {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let offset = if local { simtime::get_local_timezone_offset() * 60}
    else {0};
    let (y, m, d, h, min, _s, _w) = simtime::get_datetime(1970, ((dur.as_secs() as i64) + offset as i64) as u64);
    format!("{m}-{d:0>2}-{y:0>2} {h:0>2}:{min:0>2}")
}