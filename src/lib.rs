#![feature(path,io,fs)]

#[macro_use]
extern crate log;

extern crate "rustc-serialize" as rustc_serialize;
#[macro_use]
extern crate err;


use rustc_serialize::hex;
use std::{fs,env,io};

use std::io::Read;
use std::path::{Path, PathBuf};

static DUMMY_MAC: &'static str = "00:00:00:00:00:00";

#[derive(Debug)]
pub enum MacAddrError {
    FromHex(hex::FromHexError),
    Io(io::Error),
    Utf8(std::string::FromUtf8Error),
}

from_error! { MacAddrError => FromHex(hex::FromHexError) }
from_error! { MacAddrError => Io(io::Error) }
from_error! { MacAddrError => Utf8(std::string::FromUtf8Error) }

#[derive(Debug)]
pub enum LocalMacAddrsError {
    MacAddr(MacAddrError),
    Io(io::Error),
}

from_error! { LocalMacAddrsError => MacAddr(MacAddrError) }
from_error! { LocalMacAddrsError => Io(io::Error) }

pub fn net_dev_mac_addr(path: &Path) -> Result<Vec<u8>, MacAddrError> {
    let mut f = try!(fs::File::open(&path.join("address")));
    let s_mac = {
        let mut v = Vec::with_capacity(DUMMY_MAC.len() + 1);
        try!(f.read_to_end(&mut v));
        try!(String::from_utf8(v.into_iter()
            .filter(|b| *b != b':')
            .collect()
        ))
    };
    Ok(try!(hex::FromHex::from_hex(&s_mac[..])))
}

pub fn sysfs() -> PathBuf {
    match env::var("SYSFS_PATH") {
        Ok(p) => PathBuf::new(&p[..]),
        Err(_) => PathBuf::new("/sys")
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
pub fn local_mac_addrs() -> Vec<Vec<u8>> {
    let mut addrs : Vec<Vec<u8>> = vec![];
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

        match net_dev_mac_addr(&*net_dev) {
            Ok(a) => addrs.push(a),
            Err(e) => warn!("Could not read mac for {:?}: {:?}", net_dev, e)
        }
    }

    addrs
}

#[cfg(test)]
mod test;

