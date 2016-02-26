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

type PrefsVersion = u32;
static PREFS_FORMAT_VERSION: PrefsVersion = 1;

#[cfg(target_os="macos")]
static PREFS_DIR_NAME: &'static str = "Library/Preferences";
#[cfg(all(unix, not(target_os="macos")))]
static PREFS_DIR_NAME: &'static str = ".config";
#[cfg(windows)]
static PREFS_DIR_NAME: &'static str = "AppData/Roaming";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Pref {
    StringValue(String),
    FloatValue(f64),
    SignedValue(i64),
    UnsignedValue(u64),
}

impl<'a> From<&'a str> for Pref {
    fn from(value: &'a str) -> Pref {
        Pref::StringValue(value.to_owned())
    }
}

impl From<String> for Pref {
    fn from(value: String) -> Pref {
        Pref::StringValue(value)
    }
}

impl From<f64> for Pref {
    fn from(value: f64) -> Pref {
        Pref::FloatValue(value)
    }
}

impl From<f32> for Pref {
    fn from(value: f32) -> Pref {
        Pref::FloatValue(value as f64)
    }
}

impl From<i64> for Pref {
    fn from(value: i64) -> Pref {
        Pref::SignedValue(value)
    }
}

impl From<i32> for Pref {
    fn from(value: i32) -> Pref {
        Pref::SignedValue(value as i64)
    }
}

impl From<u64> for Pref {
    fn from(value: u64) -> Pref {
        Pref::UnsignedValue(value)
    }
}

impl From<u32> for Pref {
    fn from(value: u32) -> Pref {
        Pref::UnsignedValue(value as u64)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct PrefMap {
    version: PrefsVersion,
    map: HashMap<String, Pref>,
}

fn get_prefs_base_path() -> Option<PathBuf> {
    std::env::home_dir().map(|mut dir| {
        dir.push(PREFS_DIR_NAME);
        dir
    })
}

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

// TODO add clear(), which will delete prefs file AND recurisvely delete empty parent
//      directories UP TO the preferences root
impl PrefMap {
    pub fn new() -> Self {
        PrefMap { version: PREFS_FORMAT_VERSION, ..Default::default() }
    }
    pub fn save<S: AsRef<str>>(&self, path: S) -> Result<(), serde_json::error::Error> {
        let path = try!(path_buf_from_name(path.as_ref()));
        path.parent().map(|parent| create_dir_all(parent));
        let mut file = try!(File::create(path));
        let result = serde_json::to_writer(&mut file, self);
        if result.is_ok() {
            try!(file.flush());
        }
        result
    }
    pub fn load<S: AsRef<str>>(path: S) -> Result<Self, serde_json::error::Error> {
        let path = try!(path_buf_from_name(path.as_ref()));
        let file = try!(File::open(path));
        serde_json::from_reader(file)
    }
    pub fn inner<'a>(&'a self) -> &'a HashMap<String, Pref> {
        &self.map
    }
    pub fn inner_mut<'a>(&'a mut self) -> &'a mut HashMap<String, Pref> {
        &mut self.map
    }
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.map.contains_key(key.as_ref())
    }
    pub fn insert<S: AsRef<str>, P: Into<Pref>>(&mut self, key: S, value: P) -> Option<Pref> {
        self.map.insert(key.as_ref().to_owned(), value.into())
    }
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&Pref> {
        self.map.get(key.as_ref())
    }
    pub fn get_mut<S: AsRef<str>>(&mut self, key: S) -> Option<&mut Pref> {
        self.map.get_mut(key.as_ref())
    }
    pub fn remove<S: AsRef<str>>(&mut self, key: S) -> Option<Pref> {
        self.map.remove(key.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::PrefMap;
    static TEST_PREFIX: &'static str = "rust_user_prefs_test";
    fn gen_test_name(name: &str) -> String {
        TEST_PREFIX.to_owned() + "/" + name
    }
    fn gen_sample_prefs() -> PrefMap {
        let mut prefs = PrefMap::new();
        prefs.insert("foo", "bar");
        prefs.insert("age", 23u32);
        prefs.insert("PI", 3.0);
        prefs.insert("offset", -9);
        prefs
    }
    #[test]
    fn test_save_load() {
        let name = gen_test_name("/save_load");
        let sample = gen_sample_prefs();
        let save_result = sample.save(&name);
        println!("Save result: {:?}", save_result);
        assert!(save_result.is_ok());
        let load_result = PrefMap::load(&name);
        println!("Load result: {:?}", load_result);
        assert!(load_result.is_ok());
        assert_eq!(load_result.unwrap(), sample);
    }
}
