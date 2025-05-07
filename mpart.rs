/// Represents the implementation of `multipart/form-data` formatted data.
///
/// This will parse the source stream into an iterator over fields
/// 
///
/// # Field Exclusivity
use std::{io::{Read,}};

pub struct MPart<'a> {
    reader: &'a mut dyn Read,
    boundary: Vec<u8>,
    buffer: [u8;4096],
    read_bytes: usize,
    slice_start: usize,
    slice_end: usize,
    first: bool,
    last: bool,
}

pub struct Part {
    pub content_type : Option<String>,
    pub content_name : String,
    pub content_size: usize,
    pub content_filename: Option<String>,
    pub content: Vec<u8>,
}

impl<'a> MPart <'a> {
    pub fn from(r: &'a mut impl Read, b: &[u8]) -> Self {
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
    
    
    fn next_byte(&mut self) -> Option<u8> {
        self.slice_start +=1;
        if self.slice_start >= self.slice_end {
            let Ok(len) = self.reader.read(&mut self.buffer) else {
                return None //Err(io::Error::new(ErrorKind::UnexpectedEof, "Failure - eof"))
            };
            if len == 0 {
                return None // Err(io::Error::new(ErrorKind::UnexpectedEof, "Failure - eof"))
            }
            self.slice_start = 0;
            self.slice_end = len;
            self.read_bytes += len;
        }
        Some(self.buffer[self.slice_start])
    }
    
    fn parse_name_line(&mut self) -> Option<(String,Option<String>)> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte()?;
            
            if b == 0x0d {
                 let b2 = self.next_byte()?; 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return None
                    } else {
                        let line = String::from_utf8(temp_stor).ok()?; 
                        //eprintln!{"dispt {line}"}
                        // TODO make case insensitive
                        match line.strip_prefix("Content-Disposition: form-data; name=\"") {
                            Some(line) => {
                                let Some((name,file)) = line.split_once('"') else {
                                    return Some((String::from(""), None))
                                };
                                if !file.is_empty() {
                                    match file.strip_prefix("; filename=\"") {
                                        Some(file) => {
                                            let Some((file,_)) = file.split_once('"') else {
                                                return Some((String::from(name), None))
                                            };
                                            return Some((String::from(name), Some(String::from(file))))
                                        }
                                        None => ()//{ return return Ok((String::from(name), None))}
                                    }
                                }
                                return Some((String::from(name), None))
                            }
                            _ => return None //Err(io::Error::new(ErrorKind::Other, "Failure - no content disposition"))
                        }
                    }
                 } else {
                    if b2 ==  0x0d {
                        // should be failure
                         return None //Ok((String::from(""), None))
                    }
                     temp_stor.push(b2)
                 }
                 
            } else {
                temp_stor.push(b)
            }
        }
    }
    
    fn parse_type_line(&mut self) -> Option<String> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte()?;
            
            if b == 0x0d {
                 let b2 = self.next_byte()?; 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return None
                    } else {
                        return match String::from_utf8(temp_stor) {
                            Ok(res) => {
                                match res.strip_prefix("Content-Type: ") { // TODO make case insensitive
                                   Some(res) => Some(res.to_string()), // can be parsed further for ;charset=UTF-8
                                    _ => Some(String::new())
                                }
                            }
                            _ => Some("".to_string())
                        }
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

impl Iterator for MPart<'_> {
    type Item = Part;
    
    //let temp = std::env::var(String::from("TEMP")).unwrap();
    fn next(&mut self) -> Option<Self::Item> {
        if self.last {
            return None
        }
        if self.first {
            let b = self.next_byte()?;
            let b2 = self.next_byte()?;
            if b == 0x2D && b2 == 0x2D {
                for i in 0..self.boundary.len() {
                    if self.next_byte()? != self.boundary[i] {
                        return None
                    }
                }
            } else {
                return None
            }
            let b = self.next_byte()?;
            let b2 = self.next_byte()?;
            if b != 0x0d || b2 != 0x0a {
                // check for last
                if b == 0x2D && b2 == 0x2D {
                    let b = self.next_byte()?;
                    let b2 = self.next_byte()?;
                    if b == 0x0d && b == 0x0a {
                        return None
                    }
                }
            }
            
            self.first = false
        }
        // read and parse line after boundary
        let Some((name,filename)) =  self.parse_name_line() else {
            return None
        };
        let content_type = 
            match self.parse_type_line() {
                None => Some("text/plain".to_string()),
                Some(text) => {
                    // read empty line
                    let b = self.next_byte()?;
                    let b2 = self.next_byte()?;
                    if b != 0x0d || b2 != 0x0a {
                        // TODO can be one more line as
                        // content-transfer-encoding: quoted-printable
                        return None
                    }
                    Some(text.to_string())
                }
        };
        //eprintln!{"read content of {name}"}
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
                    if temp_stor.len() == self.boundary.len()+2 {
                        let b = self.next_byte()?;
                        let b2 = self.next_byte()?;
                        
                        if b == 0x0d && b2 == 0x0a || b == 0x2D && b2 == 0x2D {
                            if b == 0x2D && b2 == 0x2D {
                                let b = self.next_byte()?;
                                let b2 = self.next_byte()?;
                                // check they are 0d 0a
                                if b != 0x0d || b2 != 0x0a {
                                    //eprintln!{"no end line"}
                                    return None
                                }
                                self.last = true
                            }
                            // remove \r\n
                            content.truncate(content.len() - 2);
                            return Some(Part {
                                   content_type : content_type,
                                    content_name : name,
                                    content_size: content.len(),
                                    content_filename: filename,
                                    content: content
                                 })
                        } else {
                            //eprintln!{"tail after sep bndry -- {b} {b2}"}
                        }  
                    } else {
                        //eprintln!{"boundary {} found {}",self.boundary.len(), temp_stor.len()}
                    }
                } else {
                    content.push(b);
                    content.push(b2)
                }
            } else {
                content.push(b)
            }
        }
    }
}