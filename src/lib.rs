#[macro_use]
extern crate err;

#[macro_use]
extern crate nix;
extern crate libc;

use std::{fs,env,io,ffi,fmt};

use std::os::unix::prelude::AsRawFd;
use std::borrow::ToOwned;
use std::io::Read;
use std::path::PathBuf;
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

pub struct Net {
    ent: fs::DirEntry,
    name: ffi::OsString,
}

impl fmt::Debug for Net {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Net({:?})", self.name)
    }
}

static SYSFS_CLASS_NET : &'static str = "class/net";
impl Net {
    pub fn from_dirent(ent: fs::DirEntry) -> Net {
        Net { name: ent.file_name(), ent: ent }
    }

    pub fn new_all() -> io::Result<Vec<Net>> {
        let net = &sysfs().join(SYSFS_CLASS_NET);
        let dirs = try!(fs::read_dir(net));

        let mut devs = vec![];
        for net_dev_res in dirs {
            let net_dev = try!(net_dev_res);
            devs.push(Net::from_dirent(net_dev));
        }

        Ok(devs)
    }

    pub fn addr(&self) -> Result<String, DevPropError> {
        let mut p = sysfs().join(SYSFS_CLASS_NET);
        p.push(self.ent.file_name());
        p.push("address");
        let mut f = try!(fs::File::open(p));
        let mut v = Vec::with_capacity(DUMMY_MAC.len() + 1);
        try!(f.read_to_end(&mut v));
        Ok(try!(std::str::from_utf8(&v[..v.len()-1])).to_owned())
    }

    pub fn name(&self) -> &ffi::OsStr {
        &self.name
    }
}

#[cfg(test)]
mod test;

