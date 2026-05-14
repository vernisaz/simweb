use crate::template;
use std::{collections::HashMap, error::Error};

pub trait WebPage {
    /// Returns response content type
    ///
    /// text/html is used by default
    fn content_type(&self) -> &str {
        "text/html"
    }

    /// The method supposes to return a response load accordingly to the content type
    ///
    /// This method has to be implemented
    fn main_load(&self) -> Result<String, Box<dyn Error>>;

    /// any additional header including cookie set in format name:value
    ///
    /// no additional headers returned by default
    fn get_extra(&self) -> Option<Vec<(String, String)>> {
        None
    }

    /// The method can modify hashmap used for response content interpolation
    ///
    /// When no interpolation is required, the map should be cleared to avoid side effects
    fn apply_specific(
        &self,
        _page_map: &mut HashMap<&str, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Returns custom response status in a form code and description
    ///
    /// None means use the standard response: 200 Ok
    fn status(&self) -> Option<(u16, &str)> {
        None
    }

    /// Customization of an error response
    fn err_out(&self, err: Box<dyn std::error::Error>) {
        print! { "Status: {} Internal Server Error\r\n", 500 }
        print! {"Content-type: text/plain\r\n\r\n{err:?}"}
    }

    /// the method has an internal implementation
    fn show(&self) {
        // => Result<(), String>
        match self.main_load() {
            Ok(page) => {
                let mut page_items = HashMap::from([("theme", String::from(""))]);
                match self.apply_specific(&mut page_items) {
                    Ok(()) => {
                        if let Some(status) = self.status() {
                            print! { "Status: {} {}\r\n", status.0, status.1 }
                        }
                        if let Some(extra_headers) = Self::get_extra(self) {
                            for header in extra_headers {
                                print! { "{}: {}\r\n", header.0, header.1 }
                            }
                        }
                        print! {"Content-type: {}\r\n\r\n", self.content_type()};
                        print! {"{}", if page_items.is_empty() {page} else {template::interpolate(&page, &page_items)}}
                    }
                    Err(error) => Self::err_out(self, error),
                }
            }
            Err(error) => Self::err_out(self, error),
        }
    }
}
