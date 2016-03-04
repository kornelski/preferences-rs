// TODO add documentation!
// TODO choose a license

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[cfg(target_os="macos")]
static PREFS_DIR_PATH: &'static str = "Library/Preferences";
#[cfg(all(unix, not(target_os="macos")))]
static PREFS_DIR_PATH: &'static str = ".config";
#[cfg(windows)]
static PREFS_DIR_PATH: &'static str = "AppData/Roaming";

pub type PreferencesMap<T=String> = HashMap<String, T>;

pub trait PreferencesTrait {
    fn save<S>(&self, path: S) -> Result<(), serde_json::error::Error> where S: AsRef<str>;
    fn load<S>(&mut self, path: S) -> Result<(), serde_json::error::Error> where S: AsRef<str>;
}

impl<T> PreferencesTrait for T
    where T: Serialize + Deserialize
{
    fn save<S>(&self, path: S) -> Result<(), serde_json::error::Error>
        where S: AsRef<str>
    {
        let path = try!(path_buf_from_name(path.as_ref()));
        path.parent().map(|parent| create_dir_all(parent));
        let mut file = try!(File::create(path));
        let result = serde_json::to_writer(&mut file, self);
        if result.is_ok() {
            try!(file.flush());
        }
        result
    }
    fn load<S>(&mut self, path: S) -> Result<(), serde_json::error::Error>
        where S: AsRef<str>
    {
        let path = try!(path_buf_from_name(path.as_ref()));
        let file = try!(File::open(path));
        serde_json::from_reader(file).map(|val| {
            *self = val;
        })
    }
}

fn get_prefs_base_path() -> Option<PathBuf> {
    std::env::home_dir().map(|mut dir| {
        dir.push(PREFS_DIR_PATH);
        dir
    })
}

// TODO clean up strings and error messages
fn path_buf_from_name(name: &str) -> Result<PathBuf, Error> {
    if name.contains("../") || name.contains("/..") {
        return Result::Err(Error::new(ErrorKind::Other, "Invalid PrefMap name"));
    }
    let mut base_path = try!(get_prefs_base_path().ok_or(Error::new(ErrorKind::NotFound,
                                                                    "Could not find user home \
                                                                     directory for preferences \
                                                                     read/write")));
    let name_path = Path::new(name);
    if !name_path.is_relative() {
        return Result::Err(Error::new(ErrorKind::Other,
                                      "Invalid preferences name: ".to_owned() + name));
    }
    base_path.push(name_path);
    Result::Ok(base_path)
}

#[cfg(test)]
mod tests {
    use {PreferencesTrait, PreferencesMap};
    static TEST_PREFIX: &'static str = "rust_user_prefs_test";
    fn gen_test_name(name: &str) -> String {
        TEST_PREFIX.to_owned() + "/" + name
    }
    fn gen_sample_prefs() -> PreferencesMap<String> {
        let mut prefs = PreferencesMap::new();
        prefs.insert("foo".into(), "bar".into());
        prefs.insert("age".into(), "23".into());
        prefs.insert("PI".into(), "3.14".into());
        prefs.insert("offset".into(), "-9".into());
        prefs
    }
    #[test]
    fn test_save_load() {
        let name = gen_test_name("/save_load");
        let sample = gen_sample_prefs();
        let save_result = sample.save(&name);
        println!("Save result: {:?}", save_result);
        assert!(save_result.is_ok());
        let mut loaded_map = PreferencesMap::new();
        let load_result = loaded_map.load(&name);
        println!("Load result: {:?}", load_result);
        assert!(load_result.is_ok());
        assert_eq!(loaded_map, sample);
    }
}
