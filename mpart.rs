
/// Represents the implementation of `multipart/form-data` formatted data.
///
/// This will parse the source stream into an iterator over fields
/// 
///
/// # Field Exclusivity

pub struct MPart {
    reader: Read,
    boundary: String,
    buffer: [u8],
    read_bytes: usize,
    slice_start: usize,
    slice_end: usize.
    first: bool,
    last: bool,
}

pub struct Part<T> {
    content_type : Option<String>,
    content_name : String,
    content_size: usize,
    content_filename: Option<String>,
    content: T
}

impl MPart {
    pub fn from(r: Read, b: &[u8]) -> Self {
        MPart {
            reader: r,
            boundary: b,
            buffer: [0_u8; 4096],
            read_bytes: 0,
            slice_start: 0,
            slice_end: 0.
            first: true,
            last: false,
        }
    }
    
    pub consumed(&self) -> usize {
        read_bytes
    }
}

impl Iterator for MPart {
    type Item = Part;
    
    let temp = std::env::var(String::from("TEMP")).unwrap();
    pub fn next(&mut self) -> Option<Self::Item> {
        loop {
            let b = self.next_byte();
            let b2 = self.next_byte();
            if b == 0x2D && b2 == 0x2D {
                for i in 0..boundary.len() {
                    if self.next_byte() != boundary[i] {
                        return None
                    }
                }
            }
            let b = self.next_byte();
            let b2 = self.next_byte();
            if b != 0x0d || b2 != 0x0a {
                return None
            }
            if b == 0x2D && b2 == 0x2D {
                let b = self.next_byte();
                let b2 = self.next_byte();
                if b == 0x0d && b == 0x0a {
                    return None
                }
            }
            // read and parse line after boundary
            let (name,filename) = self.parse_name_line();
            let content_type = self.parse_type_line();
            let content_type = 
            match content_type {
                None => Some("text/plain"),
                Some(bytes) => {
                    let b = self.next_byte();
                    let b2 = self.next_byte();
                    if b != 0x0d || b2 != 0x0a {
                        return None
                    }
                    match String::from_utf8(bytes) {
                        Ok(content_type) => content_type,
                        _ => String::from("")
                    }
                }
            };
            let mut content = Vec::new();
            let mut temp_stor = Vec::new();
            loop {
                let b = self.next_byte();
                if b == 0x2D {
                    let b2 = self.next_byte();
                    if b2 == 0x2d {
                        temp_stor.clear();
                        temp_stor.push(b);
                        temp_stor.push(b);
                        for i in 0..boundary.len() {
                            let bn = self.next_byte();
                            if  bn != boundary[i] {
                                content.append(temp_stor);
                                temp_stor.push(bn);
                                break
                            }
                            temp_stor.push(bn)
                        }
                        if temp_stor.len() == boundary.len() {
                            let b = self.next_byte();
                            let b2 = self.next_byte();
                            
                            if b == 0x0d && b2 == 0x0a || b == 0x2D && b2 == 0x2D {
                                if b == 0x2D && b2 == 0x2D {
                                    let b = self.next_byte();
                                    let b2 = self.next_byte();
                                }
                                return Part {
                                   content_type : content_type,
                                    content_name : name,
                                    content_size: 0,
                                    content_filename: filename,
                                    content: if content_type.is_some() && content_type.unwrap().starts_with("text") {
                                        String::from_utf8_lossy(&content) } else {
                                            content
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
            break
        }
        None
    }
    
    fn parse_name_line(&mut self) -> io::Result<(String,Option<String>)> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte();
            
            if b == 0x0d {
                 let b2 = self.next_byte(); 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return Ok((String::new(), None))
                    } else {
                        let mut line = String::from_utf8(&temp_stor)?;
                        if line.starts_with("Content-Disposition: form-data; name=\"") {
                            line = line.strip_prefix("Content-Disposition: form-data; name=\"").unwrap();
                            let Some((name,file)) = line.split_once('"') else {
                                return Ok((String::from(""), None))
                            }
                            if !file.is_empty() {
                                match file.strip_prefix("; filename=\"") {
                                    Some(file) => {
                                        let (file,_) = file.split_once('"') else {
                                            return Ok((String::from(name), None))
                                        }
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
    
    fn parse_type_line(&mut self) -> io::Result<Option<[u8]>> {
        let mut temp_stor = Vec::new();
        loop {
            let b = self.next_byte();
            
            if b == 0x0d {
                 let b2 = self.next_byte(); 
                 if b2 == 0x0a {
                    if temp_stor.is_empty() {
                        return Ok(None)
                    } else {
                        return Ok(Some(*&temp_stor))
                    }
                 } else {
                     temp_stor.push(b2)
                 }
            } else {
                temp_stor.push(b)
            }
        }
    }
    
    fn next_byte(&mut self) -> Option<u8> {
        slice_start +=1;
        if slice_start == slice_end {
            let Some(len) = self.reader.read(self.buffer) else {
                return None
            };
            if len == 0 {
                return None
            }
            slice_start = 0;
            slice_end = len;
            self.read_bytes += len;
        }
        self.buffer[slice_start]
    }
}