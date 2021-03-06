use std::sync::{RwLock, Arc, Mutex};
use std::fs;
use std::ffi::{CString, CStr};
use std::str;
use libc::{c_void, c_int, size_t, c_char};

use dormin::vec;

pub type Arw<T> = Arc<RwLock<T>>;
pub type Mx<T> = Arc<Mutex<T>>;

pub fn vec3_center(pos : &[vec::Vec3]) -> vec::Vec3
{
    let mut v = vec::Vec3::zero();
    for p in pos
    {
        v = v + *p;
    }

    v = v / pos.len() as f64;

    v
}


use std::path::{Path, PathBuf};
pub fn get_files_in_dir(path : &str) -> Vec<PathBuf>
{
    let files = fs::read_dir(path).expect(&format!("missing directory : '{}'", path));
    /*
    for file in files {
        println!("Name: {}", file.unwrap().path().display())
    }
    */

    files.map(|x| x.unwrap().path()).collect()
}

#[link(name = "joker")]
extern {
    fn do_something_with_slice(slice : *const c_void, len : size_t);
}

pub fn to_cstring(v : Vec<PathBuf>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.to_str().unwrap()).unwrap()).collect()
}

pub fn string_to_cstring(v : Vec<String>) -> Vec<CString>
{
    v.iter().map(|x| CString::new(x.as_str()).unwrap()).collect()
}

pub fn print_vec_cstring(v : Vec<CString>)
{
    let y : Vec<*const c_char> = v.iter().map( |x| x.as_ptr()).collect();

    unsafe { do_something_with_slice(
            y.as_ptr() as *const c_void,
            y.len() as size_t); }
}

pub fn pass_slice() 
{
    let s = [ 
        CString::new("test").unwrap().as_ptr(),
        CString::new("caca").unwrap().as_ptr(),
        CString::new("bouda").unwrap().as_ptr() ];

    unsafe { do_something_with_slice(
            s.as_ptr() as *const c_void,
            s.len() as size_t); }
}

pub fn join_string(path : &[String]) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(v);
        first = false;
    }

    s
}

pub fn join_str(path : &[&str]) -> String
{
    let mut s = String::new();
    let mut first = true;
    for v in path {
        if !first {
            s.push('/');
        }
        s.push_str(*v);
        first = false;
    }

    s
}

