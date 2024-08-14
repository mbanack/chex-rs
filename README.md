[![MIT License][license-shield]][license-url]

<p align="center">
    <a href="https://docs.rs/...">docs</a><br/>
    <a href="https://github.com/mbanack/chex-rs/examples">examples</a><br/>
    <a href="https://github.com/mbanack/chex-rs/issues/new?labels=bug">Report a bug</a><br/>
</p>

## chex -- the async/sync check exit signal

Chex is a simple rust library which provides a global exit signal which can be easily shared between any/all threads and async tasks in a program.

This allows you to set clear policy in your program of which conditions should teardown the entire program, and make sure that all other independent tasks/threads will receive the exit signal in a somewhat timely fashion and can perform their own teardown logic before exiting.  Specifically we can avoid the cases where independent worker threads or tokio runtimes continue running after one of them Panics.

A ChexInstance can be acquired in two ways:
1. Cloned from any other ChexInstance
2. Acquired from anywhere with an associated function of the global Chex::get_chex_instance()

Usage guidelines:
1. Very early in the main task/thread call Chex::init(set_exit_on_panic: bool).  After that a ChexInstance can be obtained immediately with .get_instance() and cloned as needed, or acquired at any other point in the program without holding a reference to the original &Chex returned from init, with the associated function Chex::get_chex_instance()
2. All threads and tasks which run for a significant amount of time should periodically check whether exit has been signalled, ie as a match within a tokio::select!() block or as a poll-check within non-async forever-loops.
3. If panic!() on one thread should be caught to send the exit signal to all other ChexInstance listeners, initialize the library with Chex::init(true).  This behavior can also be enabled after the fact with Chex.set_exit_on_panic().

See the examples/ folder for usage with a mix of independent tokio runtimes and non-async worker threads.

## dependencies + justification

1. async-broadcast: async/sync channels with overflow
2. log::error: used on Panic paths only
