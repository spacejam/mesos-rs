use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::thread;

use client::SchedulerClient;
use recordio::RecordIOCodec;
use proto::mesos::FrameworkID;
use proto::scheduler::*;
use Scheduler;
use util;

pub fn run_protobuf_scheduler(master_url: String,
                              user: String,
                              name: String,
                              framework_timeout: f64,
                              scheduler: &mut Scheduler,
                              framework_id: Option<String>) {

    let mesos_framework_id =
        framework_id.map(|framework_id| {
            let mut proto_framework_id = FrameworkID::new();
            proto_framework_id.set_value(framework_id);
            proto_framework_id
        });

    let client = SchedulerClient {
        url: master_url,
        framework_id: Arc::new(Mutex::new(None)),
    };
    let client_clone = client.clone();

    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut codec = RecordIOCodec::new(tx);
        let framework_info = util::framework_info(user,
                                                  name,
                                                  framework_timeout);
        let mut res = client_clone.subscribe(framework_info, None).unwrap();
        io::copy(&mut res, &mut codec).unwrap();
    });

    for event in rx {
        match event.get_field_type() {
            Event_Type::SUBSCRIBED => {
                let subscribed = event.get_subscribed();
                let mut framework_id = client.framework_id.lock().unwrap();
                *framework_id = Some(subscribed.get_framework_id().clone());

                let heartbeat_interval_seconds =
                    if !subscribed.has_heartbeat_interval_seconds() {
                        None
                    } else {
                        Some(subscribed.get_heartbeat_interval_seconds())
                    };

                scheduler.subscribed(&client,
                                     subscribed.get_framework_id(),
                                     heartbeat_interval_seconds)
            }
            Event_Type::OFFERS => {
                let offers = event.get_offers();
                scheduler.offers(&client,
                                 offers.get_offers().to_vec(),
                                 offers.get_inverse_offers().to_vec())
            }
            Event_Type::RESCIND =>
                scheduler.rescind(&client, event.get_rescind().get_offer_id()),
            Event_Type::UPDATE =>
                scheduler.update(&client, event.get_update().get_status()),
            Event_Type::MESSAGE => {
                let message = event.get_message();
                scheduler.message(&client,
                                  message.get_agent_id(),
                                  message.get_executor_id(),
                                  message.get_data().to_vec())
            }
            Event_Type::FAILURE => {
                let failure = event.get_failure();
                let executor_id = if !failure.has_executor_id() {
                    None
                } else {
                    Some(failure.get_executor_id())
                };
                let status = if !failure.has_status() {
                    None
                } else {
                    Some(failure.get_status())
                };
                scheduler.failure(&client,
                                  failure.get_agent_id(),
                                  executor_id,
                                  status)
            }
            Event_Type::ERROR =>
                scheduler.error(&client,
                                event.get_error().get_message().to_string()),
            Event_Type::HEARTBEAT => scheduler.heartbeat(&client),
        }
    }
}
