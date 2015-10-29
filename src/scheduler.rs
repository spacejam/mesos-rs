use std::io;
use std::thread;
use std::sync::mpsc::channel;

use client::subscribe;
use recordio::RecordIOCodec;
use proto::scheduler::{Event, Event_Type};

pub trait Scheduler {
    fn received(&mut self, Event);
}

pub struct TestScheduler;

impl Scheduler for TestScheduler {
    fn received(&mut self, event: Event) {
        println!("got event in received!");
        match event.get_field_type() {
            Event_Type::SUBSCRIBED => {},
            Event_Type::OFFERS => {},
            Event_Type::RESCIND => {},
            Event_Type::UPDATE => {},
            Event_Type::MESSAGE => {},
            Event_Type::FAILURE => {},
            Event_Type::ERROR => {},
            Event_Type::HEARTBEAT => {},
        }
    }
}

struct ProtobufEventStream;

impl ProtobufEventStream {
    fn run(self,
           master_url: String,
           user: String,
           name: String,
           framework_timeout: f64,
           scheduler: &mut Scheduler) {

        let (tx, rx) = channel();
        thread::spawn(move|| {
            let mut codec = RecordIOCodec::new(tx);
            let mut res = subscribe(master_url, user, name, framework_timeout).unwrap();
            println!("Response: {}", res.status);
            println!("Headers:\n{}", res.headers);

            io::copy(&mut res, &mut codec).unwrap();
        });
        for event in rx {
            scheduler.received(event);
        }
    }
}

#[test]
fn main() {
    let url = "http://localhost:5050/api/v1/scheduler".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;

    let mut scheduler = TestScheduler;
    let event_stream = ProtobufEventStream;
    event_stream.run(url, user, name, framework_timeout, &mut scheduler);
}
