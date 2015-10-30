#![crate_id = "mesos"]
#![crate_type = "lib"]

pub mod proto;
pub mod client;
pub mod event_stream;
pub mod scheduler;
pub mod recordio;
pub mod util;

pub use client::SchedulerClient;
pub use scheduler::Scheduler;
pub use event_stream::run_protobuf_scheduler;

extern crate hyper;
extern crate protobuf;
