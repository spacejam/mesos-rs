use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use client::SchedulerClient;
use recordio::RecordIOCodec;
use proto::mesos::FrameworkID;
use proto::scheduler::*;
use util;

pub trait Scheduler {
    fn subscribed(&mut self, subscribed: &Event_Subscribed);
    fn offers(&mut self, offers: &Event_Offers);
    fn rescind(&mut self, rescind: &Event_Rescind);
    fn update(&mut self, update: &Event_Update);
    fn message(&mut self, message: &Event_Message);
    fn failure(&mut self, failure: &Event_Failure);
    fn error(&mut self, error: &Event_Error);
    fn heartbeat(&mut self) {}
}

pub struct TestScheduler;
impl Scheduler for TestScheduler {
    fn subscribed(&mut self, subscribed: &Event_Subscribed) { println!("received subscribed"); }
    fn offers(&mut self, offers: &Event_Offers) { println!("received offers"); }
    fn rescind(&mut self, rescind: &Event_Rescind) { println!("received rescind"); }
    fn update(&mut self, update: &Event_Update) { println!("received update"); }
    fn message(&mut self, message: &Event_Message) { println!("received message"); }
    fn failure(&mut self, failure: &Event_Failure) { println!("received failure"); }
    fn error(&mut self, error: &Event_Error) { println!("received error"); }
    fn heartbeat(&mut self) { println!("received heartbeat"); }
}

struct ProtobufEventStream;

impl ProtobufEventStream {
    fn run(self,
           master_url: String,
           user: String,
           name: String,
           framework_timeout: f64,
           scheduler: &mut Scheduler,
           framework_id: Option<String>) {

        let mesos_framework_id = framework_id.map(|framework_id| {
            let mut proto_framework_id = FrameworkID::new();
            proto_framework_id.set_value(framework_id);
            proto_framework_id
        });

        let client = SchedulerClient { url: master_url, framework_id: Arc::new(Mutex::new(None)) };

        let (tx, rx) = channel();
        thread::spawn(move|| {
            let mut codec = RecordIOCodec::new(tx);
            let framework_info = util::framework_info(user, name, framework_timeout);
            let mut res = client.clone().subscribe(framework_info, None).unwrap();
            io::copy(&mut res, &mut codec).unwrap();
        });

        for event in rx {
            match event.get_field_type() {
                Event_Type::SUBSCRIBED => {
                    let subscribed = event.get_subscribed();
                    let mut framework_id = client.framework_id.lock().unwrap();
                    *framework_id = Some(subscribed.get_framework_id().clone());

                    scheduler.subscribed(subscribed)
                },
                Event_Type::OFFERS => scheduler.offers(event.get_offers()),
                Event_Type::RESCIND => scheduler.rescind(event.get_rescind()),
                Event_Type::UPDATE => scheduler.update(event.get_update()),
                Event_Type::MESSAGE => scheduler.message(event.get_message()),
                Event_Type::FAILURE => scheduler.failure(event.get_failure()),
                Event_Type::ERROR => scheduler.error(event.get_error()),
                Event_Type::HEARTBEAT => scheduler.heartbeat(),
            }
        }
    }
}

#[test]
fn main() {
    let url = "http://localhost:5050/api/v1/scheduler".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;
    let framework_id = None;

    let mut scheduler = TestScheduler;
    let event_stream = ProtobufEventStream;
    event_stream.run(url, user, name, framework_timeout, &mut scheduler, framework_id);
}
