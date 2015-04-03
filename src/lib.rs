#![feature(core)]

#[macro_use]
extern crate log;

extern crate rustc_serialize;
#[macro_use]
extern crate err;


use rustc_serialize::hex;
use std::{fs,env,io};

use std::borrow::ToOwned;
use std::io::Read;
use std::path::{Path, PathBuf};

static DUMMY_MAC: &'static str = "00:00:00:00:00:00";

from_enum! {
    pub enum DevPropError {
        auto Io(io::Error),
        auto Utf8(std::str::Utf8Error)
    }
}

pub fn macaddr_from_str(s: &str) -> Result<Vec<u8>, hex::FromHexError> {
    let mac : String = s.chars()
            .filter(|b| *b != ':')
            .collect();
    hex::FromHex::from_hex(&mac[..])
}

pub fn netdev_address(path: &Path) -> Result<String, DevPropError> {
    let mut f = try!(fs::File::open(&path.join("address")));
    let s_mac = {
        let mut v = Vec::with_capacity(DUMMY_MAC.len() + 1);
        try!(f.read_to_end(&mut v));
        try!(std::str::from_utf8(&v[..v.len()-1])).to_owned()
    };
    Ok(s_mac)
}

pub fn sysfs() -> PathBuf {
    match env::var("SYSFS_PATH") {
        Ok(p) => PathBuf::from(&p[..]),
        Err(_) => PathBuf::from("/sys")
    }
}

macro_rules! try_or {
    ($try:expr, $or:expr) => (
        let t = $try;
        match t {
            Ok(v) => v,
            Err(e) => return $or(e)
        }
    )
}


/* TODO: use a full representation of the system's devices */
static SYSFS_CLASS_NET : &'static str = "class/net";
pub fn netdev_addrs() -> Vec<String> {
    let mut addrs : Vec<String> = vec![];
    let net = &sysfs().join(SYSFS_CLASS_NET);
    let dirs = match fs::read_dir(net) {
        Ok(a) => a,
        Err(e) => {
            warn!("Could not read '{:?}', {}", net, e);
            return vec![];
        }
    };

    for net_dev_res in dirs {
        let net_dev = match net_dev_res {
            Ok(a) => a.path(),
            Err(e) => {
                warn!("Error reading dir: {:?}", e);
                continue;
            }
        };

        let name = match (*net_dev).file_name() {
            Some(x) => x,
            None => {
                warn!("No filename for {:?}", net_dev);
                continue;
            }
        };

        if name == "lo" {
            continue;
        }

        match netdev_address(&*net_dev) {
            Ok(a) => addrs.push(a),
            Err(e) => warn!("Could not read mac for {:?}: {:?}", net_dev, e)
        }
    }

    addrs
}

#[cfg(test)]
mod test;

