use chex::{Chex,ChexInstance};

#[test]
fn init_chex() {
    let chex: &Chex = Chex::init(true);
    let ci: ChexInstance = chex.get_instance();

    assert_eq!(ci.poll_exit(), false);
    assert_eq!(chex.poll_exit(), false);
    chex.signal_exit();
    assert_eq!(ci.poll_exit(), true);
    assert_eq!(chex.poll_exit(), true);
}
