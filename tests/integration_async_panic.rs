use chex::{Chex,ChexInstance};

#[should_panic]
fn thread_one() {
    panic!("thread_one panic");
}

async fn task_two() {
    let mut ci = Chex::get_chex_instance();

    ci.check_exit_async().await;
    println!("tokio task_two got exit signal");
}

fn thread_two() {
    let tk_runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tk_runtime.block_on(async move {
        task_two().await;
    });
}

fn thread_three(chex: ChexInstance) {
    while !chex.poll_exit() { }
    println!("thread_three got exit signal");
}

#[test]
fn multi_runtime_task_panic() {
    let chex: &Chex = Chex::init(false);
    assert!(!chex.poll_exit());
    chex.set_exit_on_panic();
    assert!(!chex.poll_exit());

    println!("main thread starting some other threads");

    let th_one = std::thread::Builder::new().spawn({
        move || {
            let res = std::panic::catch_unwind(|| {
                thread_one();
            });

            assert!(res.is_err());
        }
    }).expect("Failed to spawn thread");

    let th_two = std::thread::Builder::new().spawn({
        move || {
            thread_two();
        }
    }).expect("Failed to spawn thread");

    let ci = chex.get_instance();
    let th_three = std::thread::Builder::new().spawn({
        move || {
            thread_three(ci);
        }
    }).expect("Failed to spawn thread");

    while !chex.poll_exit() { }

    println!("main thread got exit signal");

    let _ = th_one.join();
    let _ = th_two.join();
    let _ = th_three.join();

    assert!(chex.poll_exit());
    let ci = chex.get_instance();
    assert!(ci.poll_exit());
}
