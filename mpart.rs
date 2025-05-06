/// Represents the implementation of `multipart/form-data` formatted data.
///
/// This will parse the source stream into an iterator over fields
/// 
///
/// # Field Exclusivity
use std::{io::{Read,self,ErrorKind,Error}};

pub struct MPart {
    reader: & dyn Read,
    boundary: Vec<u8>,
    buffer: [u8;4096],
    read_bytes: usize,
    slice_start: usize,
    slice_end: usize,
    first: bool,
    last: bool,
}

pub struct Part<T> {
    pub content_type : Option<String>,
    pub content_name : String,
    pub content_size: usize,
    pub content_filename: Option<String>,
    pub content: T
}

impl MPart {
    pub fn from(r: impl Read, b: &[u8]) -> Self {
        MPart {
            reader: r,
            boundary: b.to_vec(),
            buffer: [0_u8; 4096],
            read_bytes: 0,
            slice_start: 0,
            slice_end: 0,
            first: true,
            last: false,
        }
    }
    
    pub fn consumed(&self) -> usize {
        self.read_bytes
    }
    
    
    fn next_byte(&mut self) -> io::Result<u8> {
        self.slice_start +=1;
        if self.slice_start == self.slice_end {
            let Some(len) = self.reader.read(self.buffer) else {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "Failure - eof"))
            };
            if len == 0 {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "Failure - eof"))
            }
            self.slice_start = 0;
            self.slice_end = len;
            self.read_bytes += len;
        }
        Ok(self.buffer[self.slice_start])
    }
    
    fn parse_name_line(&mut self) -> io::Result<(String,Option<String>)> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte()?;
            
            if b == 0x0d {
                 let b2 = self.next_byte()?; 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return Ok((String::new(), None))
                    } else {
                        let mut line = String::from_utf8(temp_stor).unwrap();//err_map()?;
                        if line.starts_with(&"Content-Disposition: form-data; name=\"".to_string()) {
                            line = line.strip_prefix(&"Content-Disposition: form-data; name=\"".to_string()).unwrap().to_string();
                            let Some((name,file)) = line.split_once('"') else {
                                return Ok((String::from(""), None))
                            };
                            if !file.is_empty() {
                                match file.strip_prefix("; filename=\"") {
                                    Some(file) => {
                                        let Some((file,_)) = file.split_once('"') else {
                                            return Ok((String::from(name), None))
                                        };
                                        return Ok((String::from(name), Some(String::from(file))))
                                    }
                                    None => ()//{ return return Ok((String::from(name), None))}
                                }
                            }
                            return Ok((String::from(name), None))
                        }
                        return Err(io::Error::new(ErrorKind::Other, "Failure - no content disposition"))
                    }
                   
                 } else {
                    if b2 ==  0x0d {
                        // should be failure
                         return Ok((String::from(""), None))
                    }
                     temp_stor.push(b2)
                 }
                 
            } else {
                temp_stor.push(b)
            }
        }
    }
    
    fn parse_type_line(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte()?;
            
            if b == 0x0d {
                 let b2 = self.next_byte()?; 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return Ok(None)
                    } else {
                        return Ok(Some(temp_stor))
                    }
                 } else {
                     temp_stor.push(b2)
                 }
            } else {
                temp_stor.push(b)
            }
        }
    }
    
}

impl Iterator for MPart {
    type Item = Part<T>;
    
    //let temp = std::env::var(String::from("TEMP")).unwrap();
    fn next(&mut self) -> Option<Self::Item> {
        loop {
        if self.first {
            let b = self.next_byte()?;
            let b2 = self.next_byte()?;
            if b == 0x2D && b2 == 0x2D {
                for i in 0..self.boundary.len() {
                    if self.next_byte()? != self.boundary[i] {
                        return None
                    }
                }
            }
            let b = self.next_byte()?;
            let b2 = self.next_byte()?;
            if b != 0x0d || b2 != 0x0a {
                return None
            }
            if b == 0x2D && b2 == 0x2D {
                let b = self.next_byte()?;
                let b2 = self.next_byte()?;
                if b == 0x0d && b == 0x0a {
                    return None
                }
            }
            self.first = false
        }
            // read and parse line after boundary
            let Ok((name,filename)) =  self.parse_name_line() else {
                return None
            };
            let content_type = 
            match self.parse_type_line() {
                Ok(content_type) => match content_type {
                    None => Some("text/plain".to_string()),
                    Some(bytes) => {
                        let b = self.next_byte()?;
                        let b2 = self.next_byte()?;
                        if b != 0x0d || b2 != 0x0a {
                            return None
                        }
                        match String::from_utf8(bytes) {
                            Ok(content_type) => Some(content_type),
                            _ => Some(String::from(""))
                        }
                    }
                }
                Err(err) => return None
            };
            let mut content = Vec::new();
            let mut temp_stor = Vec::new();
            loop {
                let b = self.next_byte()?;
                if b == 0x2D {
                    let b2 = self.next_byte()?;
                    if b2 == 0x2d {
                        temp_stor.clear();
                        temp_stor.push(b);
                        temp_stor.push(b);
                        for i in 0..self.boundary.len() {
                            let bn = self.next_byte()?;
                            if  bn != self.boundary[i] {
                                content.append(&mut temp_stor);
                                content.push(bn);
                                break
                            }
                            temp_stor.push(bn)
                        }
                        if temp_stor.len() == self.boundary.len() {
                            let b = self.next_byte()?;
                            let b2 = self.next_byte()?;
                            
                            if b == 0x0d && b2 == 0x0a || b == 0x2D && b2 == 0x2D {
                                if b == 0x2D && b2 == 0x2D {
                                    let b = self.next_byte()?;
                                    let b2 = self.next_byte()?;
                                    // check they are 0d 0a
                                }
                                return
                                if content_type.is_some() && content_type.unwrap().starts_with("text") {
                                     Part {
                                       content_type : content_type,
                                        content_name : name,
                                        content_size: 0,
                                        content_filename: filename,
                                        content: String::from_utf8_lossy(&content) ,
                                     }
                                } else {
                                      Part {
                                       content_type : content_type,
                                        content_name : name,
                                        content_size: 0,
                                        content_filename: filename,
                                        content: content
                                     }
                                 }
                               }
                               
                            }
                        } else {
                             content.push(b2)
                        }
                } else {
                    content.push(b)
                }
            }
            //break
        }
        //None
    }
    
    
}