# Crate simweb

## Purpose

Simplify writing Rust CGI scripts

## How to use

The create provides a trait simplifying writing web applications. 

1. create a structure which represents a data of http response
2. implement `WebPage` for the structure. Generally only `main_load` is required to be implemented
3. call `show` for the structure specified in step 1

Since `show` can propagate an error, its result has to be returned in `Ok` method of `main`.

Consider web **hello world** as:

```
use simweb::FiveXXError;
use simweb::WebPage;

struct Hello;

fn main()  -> Result<(), FiveXXError> {
   Ok(Hello{}.show()) 
}

impl simweb::WebPage for Hello {
    fn main_load(&self) -> Result<String, String> {
        Ok(r#"<!doctype html>
<html><body>Hello web world</body></html>"#.to_string ())
    }
}
```

See [test](https://github.com/vernisaz/simweb/blob/master/test/test.rs) as a more complex example.

## Dependencies

**simtime**