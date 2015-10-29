#[test]
fn show_netdev_addrs() {
    let mut n = super::Net::new_all().unwrap();
    n.sort_by(|a, b| a.name().cmp(b.name()));
    println!("{:?}", n)
}
