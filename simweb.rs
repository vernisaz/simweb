use std::collections::HashMap;
use crate::template;

pub trait WebPage {
    fn content_type(&self) -> String {
        "text/html".to_string()
    }

    fn main_load(&self) -> Result<String, String>;
    
    // any additional header including cookie set
    fn get_extra(&self) -> Option<Vec<(String, String)>> {None}

    fn apply_specific(&self, _page_map: &mut HashMap<String, String>) -> Result<(), String> { Ok(())}
    
    fn status(&self) -> Option<(u16, &str)> {
        None
    }
    
    fn err_out(&self, err: String) {
        print!{ "Status: {} Internal Server Error\r\n", 500 }
        print! {"Content-type: text/plain\r\n\r\n{err}"}
    }

    fn show(&self) { // => Result<(), String>
        match self.main_load() { 
            Ok(page) => {
                if let Some(status) = self.status() {
                    print!{ "Status: {} {}\r\n", status.0, status.1 }
                }
                if let Some(extra_headers) = Self::get_extra(&self) {
                    for header in extra_headers {
                        print!{ "{}: {}\r\n", header.0, header.1 }
                    }
                }
                print! {"Content-type: {}\r\n\r\n", self.content_type()};
                let mut page_items = HashMap::from([
                    ("theme".to_string(), String::from("")),
                ]);
                match self.apply_specific(&mut page_items) { 
                    Ok(()) => print! {"{}", template::interpolate(&page, &page_items)},
                    Err(error) => Self::err_out(&self, error)
                }
            }
            Err(error) => Self::err_out(&self, error)
        }
    }
}