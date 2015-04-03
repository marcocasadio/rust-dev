extern crate env_logger;

#[test]
fn show_netdev_addrs() {
    or_panic!(env_logger::init());
    println!("{:?}", super::netdev_addrs())
}

#[test]
fn macaddr_from_str() {
    assert_eq!(super::macaddr_from_str("00:11:22:33").unwrap(), [ 0, 0x11, 0x22, 0x33 ]);
}
