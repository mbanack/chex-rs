use chex::Chex;

#[test]
fn init_chex_global_only() {
    let chex: &Chex = Chex::init(true);

    assert_eq!(chex.poll_exit(), false);
    chex.signal_exit();
    assert_eq!(chex.poll_exit(), true);
}
