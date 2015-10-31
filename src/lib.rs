#![crate_id = "mesos"]
#![crate_type = "lib"]

pub mod executor;
pub mod executor_client;
pub mod executor_event_stream;
pub mod proto;
pub mod recordio;
pub mod scheduler;
pub mod scheduler_client;
pub mod scheduler_event_stream;
pub mod util;

pub use executor::Executor;
pub use executor_client::ExecutorClient;
pub use executor_event_stream::run_protobuf_executor;
pub use scheduler::Scheduler;
pub use scheduler_client::SchedulerClient;
pub use scheduler_event_stream::run_protobuf_scheduler;

extern crate hyper;
extern crate protobuf;
extern crate itertools;
