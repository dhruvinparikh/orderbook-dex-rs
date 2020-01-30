//! A `CodeExecutor` specialization which uses natively compiled runtime when the wasm to be
//! executed is equivalent to the natively compiled code.

use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;

native_executor_instance!(
    pub Executor,
    node_runtime::api::dispatch,
    node_runtime::native_version
);
