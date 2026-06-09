use crate::template;
use std::{collections::HashMap, error::Error};

pub trait WebPage {
    /// Returns a response content type
    ///
    /// text/html is used by default
    fn content_type(&self) -> &str {
        "text/html"
    }

    /// The method supposes to return a response load accordingly to the content type
    ///
    /// This method has to be implemented
    fn main_load(&self) -> Result<String, Box<dyn Error>>;

    /// Returns a vec of additional headers including cookie set in format name:value
    ///
    /// no additional headers returned by default
    fn get_extra(&self) -> Option<Vec<(String, String)>> {
        None
    }

    /// The method can modify hashmap used for a response content interpolation
    ///
    /// When no interpolation is required, the map should be cleared to avoid side effects.
    /// If an error happens during applying effects, a response with this error will be returned.
    fn apply_specific(&self, _page_map: &mut HashMap<&str, String>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Returns custom response status as a tuple: (code and description)
    ///
    /// None means use the standard response: 200 Ok
    fn status(&self) -> Option<(u16, &str)> {
        None
    }

    /// Outs an error response
    ///
    /// The method can be implemented for a response customization.
    fn err_out(&self, err: Box<dyn Error>) {
        print! { "Status: {} Internal Server Error\r\n", 500 }
        print! {"Content-type: text/plain\r\n\r\n{err:?}"}
    }

    /// The method has an internal implementation
    fn show(&self) {
        // => Result<(), String>
        match self.main_load() {
            Ok(page) => {
                let mut page_items = HashMap::from([("theme", String::new())]);
                match self.apply_specific(&mut page_items) {
                    Ok(_) => {
                        if let Some(status) = self.status() {
                            print! { "Status: {} {}\r\n", status.0, status.1 }
                        }
                        if let Some(extra_headers) = self.get_extra() {
                            for header in extra_headers {
                                print! { "{}: {}\r\n", header.0, header.1 }
                            }
                        }
                        print! {"Content-type: {}\r\n\r\n", self.content_type()};
                        print! {"{}", if page_items.is_empty() {page} else {template::interpolate(&page, &page_items)}}
                    }
                    Err(error) => self.err_out(error),
                }
            }
            Err(error) => self.err_out(error),
        }
    }
}
