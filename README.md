# Crate simweb

## Purpose

Simplify writing Rust CGI scripts

## How to use

The create provides a trait simplifying writing web applications. 

1. create a structure which represents a data of http response
2. implement `WebPage` for the structure. Generally only `main_load` is required to be implemented
3. call `show` for the structure specified in step 1

Since `show` can propagate an error, its result has to be returned in `Ok` method of `main`.


See [test](https://github.com/vernisaz/simweb/blob/master/test/test.rs).

## Dependencies

**simtime**