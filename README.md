# Crate simweb

## Purpose

Simplify writing Rust CGI scripts

## How to use

The crate provides a trait simplifying writing web applications. 

1. create a structure which represents a data of http response
2. implement `WebPage` for the structure. Generally only `main_load` is required to be implemented
3. call `show` for the structure specified in step 1

Consider web **hello, world** as:

```rust
use simweb::WebPage;

struct Hello;

fn main() {
   Hello{}.show()
}

impl simweb::WebPage for Hello {
    fn main_load(&self) -> Result<String, String> {
        Ok(r#"<!doctype html>
<html><body>Hello, web world</body></html>"#.to_string ())
    }
}
```

See [test](https://github.com/vernisaz/simweb/blob/master/test/test.rs) for a more sophisticated example.

## Features
The crate supports POST for forms and multi-part forms.

## Dependencies

**simtime**
## Version 
The version is 1.01

## Building the crate

Use Cargo (creation of TOML file is required), or RustBee ([bee.7b](https://github.com/vernisaz/simweb/blob/master/bee.7b) is provided).