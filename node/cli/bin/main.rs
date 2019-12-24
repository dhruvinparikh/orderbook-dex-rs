//! DNA node executable.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

use futures::channel::oneshot;
use futures::{future, FutureExt};
use sc_cli::VersionInfo;

use std::cell::RefCell;

// handles ctrl-c
struct Exit;
impl sc_cli::IntoExit for Exit {
    type Exit = future::Map<oneshot::Receiver<()>, fn(Result<(), oneshot::Canceled>) -> ()>;
    fn into_exit(self) -> Self::Exit {
        // can't use signal directly here because CtrlC takes only `Fn`.
        let (exit_send, exit) = oneshot::channel();

        let exit_send_cell = RefCell::new(Some(exit_send));
        ctrlc::set_handler(move || {
            if let Some(exit_send) = exit_send_cell.try_borrow_mut().expect("signal handler not reentrant; qed").take() {
                exit_send.send(()).expect("Error sending exit notification");
            }
        }).expect("Error setting Ctrl-C handler");

        exit.map(|_| ())
    }
}

fn main() -> Result<(), sc_cli::error::Error> {
    let version = VersionInfo {
        name: "DNA Node - Metaverse",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "dnaruntime",
        author: "BlockX Labs <info@blockxlabs.com>",
        description: "Substrate based implementation of DNA Network",
        support_url: "N/A",
    };

    node_cli::run(std::env::args(), Exit, version)
}
