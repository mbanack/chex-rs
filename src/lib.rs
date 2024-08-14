//! Global signal/check exit to coordinate multiple tasks or threads.
//!
//! Can be used in async and sync contexts.
//!
//! This allows you to set clear policy in your program of which conditions should teardown the entire program, and make sure that all other independent tasks/threads will receive the exit signal in a somewhat timely fashion and can perform their own teardown logic before exiting.  Specifically we can avoid the cases where independent worker threads or tokio runtimes continue running after one of them Panics.
//!
//! A ChexInstance can be acquired in two ways:
//! 1. Cloned from any other ChexInstance
//! 2. Acquired from anywhere with an associated function of the global Chex::get_chex_instance()
//!
//! ## Usage guidelines:
//! 1. Very early in the main task/thread call Chex::init(set_exit_on_panic: bool).  After that a ChexInstance can be obtained immediately with .get_instance() and cloned as needed, or acquired at any other point in the program without holding a reference to the original &Chex returned from init, with the associated function Chex::get_chex_instance()
//! 2. All threads and tasks which run for a significant amount of time should periodically check whether exit has been signalled, ie as a match within a tokio::select!() block or as a poll-check within non-async forever-loops.
//! 3. If panic!() on one thread should be caught to send the exit signal to all other ChexInstance listeners, initialize the library with Chex::init(true).  This behavior can also be enabled after the fact with Chex.set_exit_on_panic().
//!
//! See the examples/ folder for usage with a mix of independent tokio runtimes and non-async worker threads.
//!
//! ## Basic usage example
//! ```
//! use chex::{Chex,ChexInstance};
//!
//! let chex: &Chex = Chex::init(true);
//! let ci_a: ChexInstance = Chex::get_chex_instance();
//! let ci_b: ChexInstance = chex.get_instance();
//!
//! ci_a.signal_exit();
//!
//! assert!(ci_b.poll_exit());
//! let ci_c = chex.get_instance();
//! assert!(ci_c.poll_exit());
//! ```

use log::error;
use std::sync::{Arc,OnceLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

static GLOBAL_CHECK_EXIT: Chex = Chex::const_default();

type ChexPanicHandler = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static>;

/*
 * Global handle to wrap ChexInstance.
 */
pub struct Chex {
    cell: OnceLock<ChexInstance>,
    default_panic_handler: OnceLock<ChexPanicHandler>,
}

/*
 * Channel wrapper for exit notifications.
 */
#[derive(Clone)]
pub struct ChexInstance {
    exit: Arc<AtomicBool>,
    chs_bcast: async_broadcast::Sender::<()>,
    chr_bcast: async_broadcast::Receiver::<()>,
}

impl Chex {
    const fn const_default() -> Self {
        Self {
            default_panic_handler: OnceLock::new(),
            cell: OnceLock::new(),
        }
    }

    /// Initialize global exit-signal state.
    /// Must be called before any other crate functions.
    ///
    /// If set_exit_on_panic is true, we will immediately register a panic hook
    /// to signal exit to all other Chex/ChexInstance listeners.  This can be enabled later with
    /// .set_exit_on_panic()
    pub fn init(set_exit_on_panic: bool) -> &'static Chex {
        let _inst = GLOBAL_CHECK_EXIT.cell.get_or_init(ChexInstance::new);

        GLOBAL_CHECK_EXIT.default_panic_handler.get_or_init(|| std::panic::take_hook());

        if set_exit_on_panic {
            GLOBAL_CHECK_EXIT.set_exit_on_panic();
        }

        &GLOBAL_CHECK_EXIT
    }

    /// Setup a panic hook to signal exit to other threads.
    /// This is called automatically if initialized with init(set_exit_on_panic = true)
    pub fn set_exit_on_panic(&self) {
        std::panic::set_hook(Box::new(|info| {
            error!("PANIC: {info}");
            error!("PANIC: signal exit to all Chex listeners");

            GLOBAL_CHECK_EXIT.signal_exit();

            /*
             * TODO: Store a list of threads that have cloned the ChexInstance and not yet
             *       dropped it, and spin here until timeout or the list length hits 1
             *       (which probably means this Panicking thread is the last holdout)
             *       and then std::process::exit(1) / abort() or just call default_handler to
             *       trigger nested panic
             *
             *       ... async-broadcast also has .sender_count / .receiver_count()
             */

            /*
             * Invoke the default panic handler.
             */
            let default_handler = GLOBAL_CHECK_EXIT.default_panic_handler.get()
                .expect("PANIC (nested): Failed to initialize Chex before Panic encountered");
            error!("PANIC: calling default panic handler");
            default_handler(info);
        }));
    }

    /// Returns an instance of the underlying ChexInstance that can be used to asynchronously check
    /// exit.
    pub fn get_instance(&self) -> ChexInstance {
        self.cell.get()
            .expect("Failed to initialize Chex before .get_instance()")
            .clone()
    }

    /// Returns an instance of the underlying ChexInstance that can be used to asynchronously check
    /// exit.
    pub fn get_chex_instance() -> ChexInstance {
        GLOBAL_CHECK_EXIT.cell.get()
            .expect("Failed to initialize Chex before .get_instance()")
            .clone()
    }

    /// Returns true iff exit has been signalled.
    pub fn poll_exit(&self) -> bool {
        let c: &ChexInstance = self.cell.get().expect("Failed to initialize Chex before .poll_exit()");
        c.exit.load(Relaxed)
    }

    /// Signal all listeners to exit, then return to allow the caller to do their own cleanup.
    ///
    /// Exits the process with a failure code if we were unable to signal exit.
    pub fn signal_exit(&self) {
        let c: Option<&ChexInstance> = self.cell.get();
        match c {
            None => {
                error!("Failed to initialize Chex before .signal_exit()");
                std::process::exit(1);
            }
            Some(c) => {
                c.signal_exit();
            }
        }
    }
}

impl ChexInstance {
    /// Initialize the channels and exit flag.
    ///
    /// Should not be called directly by library users.
    fn new() -> Self {
        let (mut chs_bcast, chr_bcast) = async_broadcast::broadcast::<()>(1);
        chs_bcast.set_overflow(true);
        Self {
            exit: Arc::new(AtomicBool::new(false)),
            chs_bcast,
            chr_bcast,
        }
    }

    /// Signal all listeners to exit, then return to allow the caller to do their own cleanup.
    ///
    /// Exits the process with a failure code if we were unable to signal exit.
    pub fn signal_exit(&self) {
        self.exit.store(true, Relaxed);

        if let Err(e) = self.chs_bcast.try_broadcast(()) {
            /*
             * This can only happen if the channel is closed or full.  Let's just exit.
             */
            error!("signal_exit failed to send broadcast: {e:?}");
            std::process::exit(1);
        }
    }

    /// Returns true iff exit has already been signalled
    pub fn poll_exit(&self) -> bool {
        self.exit.load(Relaxed)
    }

    /// Returns when exit has been signalled, or the exit-signal channel is closed.
    pub async fn check_exit_async(&mut self) {
        let ex = self.exit.load(Relaxed);
        if ex {
            return;
        }

        let _ = self.chr_bcast.recv().await;
    }
}
