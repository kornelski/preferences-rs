#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::env::home_dir;
use std::path::{Path, PathBuf};

static PREFS_DIR_NAME: &'static str = ".rs_user_prefs";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Pref {
    StringValue(String),
    FloatValue(f64),
    SignedValue(i64),
    UnsignedValue(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PrefMap {
    map: HashMap<String, Pref>,
}

// TODO more validation of key
fn path_buf_from_key(key: &str) -> Result<PathBuf, Error> {
    let mut home_path = try!(
        home_dir().ok_or(
            Error::new(
                ErrorKind::NotFound,
                "Could not find user home directory for preferences read/write"
            )
        )
    );
    let key_path = Path::new(key);
    if !key_path.is_relative() {
        return Result::Err(
            Error::new(ErrorKind::Other, "Invalid preferences key: ".to_owned() + key)
        );
    }
    home_path.push(PREFS_DIR_NAME);
    home_path.push(key_path);
    Result::Ok(home_path)
}

impl PrefMap {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn save(&self, key: &str) -> Result<(), serde_json::error::Error> {
        let path = try!(path_buf_from_key(key));
        let mut file = try!(File::create(path));
        serde_json::to_writer(&mut file, self)
    }
    pub fn load(key: &str) -> Result<Self, serde_json::error::Error> {
        let path = try!(path_buf_from_key(key));
        let file = try!(File::open(path));
        serde_json::from_reader(file)
    }
}

#[cfg(test)]
mod tests {
    
}