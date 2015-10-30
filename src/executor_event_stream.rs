use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use executor_client::ExecutorClient;
use recordio::RecordIOCodec;
use proto::mesos::FrameworkID;
use proto::executor::*;
use Executor;
use util;

// TODO(tyler) parse env vars, reuse code from
// scheduler_event_stream
pub fn run_protobuf_executor() {
}
