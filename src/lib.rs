#![crate_id = "mesos"]
#![crate_type = "lib"]

pub mod proto;
pub mod client;
pub mod scheduler;
pub mod recordio;

extern crate hyper;
extern crate protobuf;
