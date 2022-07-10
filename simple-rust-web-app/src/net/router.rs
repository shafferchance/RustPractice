use std::collections::HashMap;
use std::pin::Pin;
use std::str::from_utf8;

const GET: &[u8; 3] = b"GET";
const POST: &[u8; 4] = b"POST";
const PUT: &[u8; 3] = b"PUT";
const DELETE: &[u8; 6] = b"DELETE";
const PATCH: &[u8; 5] = b"PATCH";

pub struct Req<'a> {
    pub path: &'a str,
    pub method: &'a str,
}

pub struct Res<'a> {
    status: &'a str,
    message: &'a str,
}

pub struct Router<'a> {
    routes: HashMap<&'a str, Pin<Box<Box<dyn Fn(Req, Res) -> ()>>>>,
}

impl<'a> Req<'a> {
    pub fn new(buffer: &'a [u8; 1024]) -> Req<'a> {
        let mut possible_match = false;
        let mut iter = buffer.split(|found_char| {
            if found_char == &b'\r' && !possible_match {
                possible_match = true;
            } else if found_char == &b'\n' && possible_match {
                possible_match = false;
                return true;
            }

            false
        }).map(|entry| {
            let (lhs, _) = entry.split_at(entry.len() - 1);
            lhs
        });
        let mut method_string_iter = iter.next().unwrap().split(|found_char| found_char == &b' ');
        let method = match method_string_iter.next() {
            Some(i) if i.starts_with(GET) => Ok("GET"),
            Some(i) if i.starts_with(POST) => Ok("POST"),
            Some(i) if i.starts_with(DELETE) => Ok("DELETE"),
            Some(i) if i.starts_with(PUT) => Ok("PUT"),
            Some(i) if i.starts_with(PATCH) => Ok("PATCH"),
            _ => Err("Invalid HTTP Method recieved"),
        }.unwrap();

        let path = match from_utf8(method_string_iter.next().unwrap()) {
            Ok(res) => res,
            Err(err) => panic!("{}", err),
        };

        Req {
            path,
            method,
        }
    }
}

impl<'a> Router<'a> {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, method: &'a str, closure: Box<dyn Fn(Req, Res) -> ()>) {
        self.routes.insert(method, Box::pin(closure));
    }
}