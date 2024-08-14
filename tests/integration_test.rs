use chex::{Chex,ChexInstance};

#[test]
fn init_chex() {
    let chex: &Chex = Chex::init(true);
    let ci: ChexInstance = chex.get_instance();

    assert!(!ci.poll_exit());
    assert!(!chex.poll_exit());
    chex.signal_exit();
    assert!(ci.poll_exit());
    assert!(chex.poll_exit());
}
