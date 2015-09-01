#[macro_use]
extern crate log;

extern crate rustc_serialize;
#[macro_use]
extern crate err;

#[macro_use]
extern crate nix;
extern crate libc;

use rustc_serialize::hex;
use std::{fs,env,io};

use std::os::unix::prelude::AsRawFd;
use std::borrow::ToOwned;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::mem;
use libc::{size_t, c_int};

static DUMMY_MAC: &'static str = "00:00:00:00:00:00";

from_enum! {
    pub enum DevPropError {
        auto Io(io::Error),
        auto Utf8(std::str::Utf8Error)
    }
}

fn nix_to_io<T>(x: nix::Result<T>) -> io::Result<T> {
    x.map_err(|v| match v {
        nix::Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
        nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, "InvalidPath"),
    })
}

/// Size in bytes
ioctl!{bad blkgetsize64 with ior!(0x12, 114, mem::size_of::<size_t>()) }
pub fn blockdev_size(fd: &AsRawFd) -> io::Result<u64> {
    unsafe {
        let mut r : u64 = 0;
        try!(nix_to_io(blkgetsize64(fd.as_raw_fd(), mem::transmute(&mut r))));
        Ok(r)
    }
}

ioctl!{none blkpbszget with 0x12, 123}
pub fn blockdev_phys_block_size(fd: &AsRawFd) -> io::Result<c_int> {
    unsafe {
        nix_to_io(blkpbszget(fd.as_raw_fd()))
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
    match env::var_os("SYSFS_PATH") {
        Some(p) => PathBuf::from(&p),
        None    => PathBuf::from("/sys")
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

