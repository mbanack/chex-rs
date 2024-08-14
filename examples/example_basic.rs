use chex::{Chex,ChexInstance};

fn main() {
    let chex: &Chex = Chex::init(true);
    let ci_a: ChexInstance = Chex::get_chex_instance();
    let ci_b: ChexInstance = chex.get_instance();

    ci_a.signal_exit();

    assert!(ci_b.poll_exit());
    let ci_c = chex.get_instance();
    assert!(ci_c.poll_exit());
}
