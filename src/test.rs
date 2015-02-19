extern crate env_logger;

#[test]
fn test_netdev() {
    or_panic!(env_logger::init());
    println!("{:?}", super::local_mac_addrs())
}
