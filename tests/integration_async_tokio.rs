use chex::{Chex,ChexInstance};
use tokio::task::JoinSet;

#[tokio::test]
//#[should_panic]
async fn tokio_tasks_signal_exit() {
    let chex: &Chex = Chex::init(false);
    assert!(!chex.poll_exit());
    //chex.set_exit_on_panic();
    assert!(!chex.poll_exit());

    let mut set = JoinSet::new();

    let ci: ChexInstance = chex.get_instance();
    set.spawn(async move {
        println!("task one looping");
        while !ci.poll_exit() {
            tokio::task::yield_now().await;
        }
        println!("task one exit");
    });

    let mut ci: ChexInstance = chex.get_instance();
    set.spawn(async move {
        println!("task two waiting for check_exit_async()");
        ci.check_exit_async().await;
    });

    let ci: ChexInstance = chex.get_instance();
    set.spawn(async move {
        /*
        let panic_fut = async {
            println!("task three panic");
            panic!("test panic");
        };

        let res = panic_fut.catch_unwind().await;
        println!("caught panic...");
        assert!(res.is_err());
        */

        println!("task three signal_exit()");
        ci.signal_exit();
    });

    println!("joining tasks...");
    while let Some(res) = set.join_next().await {
        println!("joined: {res:?}");
    }

    /*
    let handle = tokio::runtime::Handle::current();

    let block_res = std::panic::catch_unwind(||  {
        handle.block_on(async move {
            println!("task three panic");
            panic!("test panic");
        });
    });
    assert!(block_res.is_err());
    */


    //while !chex.poll_exit() { }
    //
    println!("done joining tasks...");

    assert!(chex.poll_exit());
    let ci = chex.get_instance();
    assert!(ci.poll_exit());
}
