use chex::Chex;

#[test]
fn init_chex_global_only() {
    let chex: &Chex = Chex::init(true);

    assert!(!chex.poll_exit());
    chex.signal_exit();
    assert!(chex.poll_exit());
}
