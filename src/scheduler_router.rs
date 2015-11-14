use std::io::{self, Error, ErrorKind, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use itertools::Itertools;

use scheduler_client::SchedulerClient;
use recordio::RecordIOCodec;
use proto::mesos::{FrameworkID, Offer};
use proto::scheduler::*;
use {Scheduler, SchedulerConf, util};

pub trait SchedulerRouter {
    fn run(&mut self,
           rx: Receiver<io::Result<Event>>,
           client: SchedulerClient,
           conf: SchedulerConf);
}

enum State {
    Connected,
    Disconnected,
}

impl State {
    fn is_connected(&self) -> bool {
        match *self {
            State::Connected => true,
            _ => false,
        }
    }
}

pub struct ProtobufCallbackRouter<'a> {
    pub scheduler: &'a mut Scheduler,
    pub conf: SchedulerConf,
}

impl <'a> SchedulerRouter for ProtobufCallbackRouter<'a> {
    fn run(&mut self,
           rx: Receiver<io::Result<Event>>,
           client: SchedulerClient,
           conf: SchedulerConf) {
        let mut state = State::Connected;
        for e in rx {
            if e.is_err() {
                if state.is_connected() {
                    state = State::Disconnected;
                    self.scheduler.disconnected();
                }
                continue;
            }
            state = State::Connected;

            let event = e.unwrap();

            match event.get_field_type() {
                Event_Type::SUBSCRIBED => {
                    let subscribed = event.get_subscribed();
                    {
                        let mut framework_id = client.framework_id
                                                     .lock()
                                                     .unwrap();
                        *framework_id = Some(subscribed.get_framework_id()
                                                       .clone());
                    }

                    let heartbeat_interval_seconds =
                        if !subscribed.has_heartbeat_interval_seconds() {
                            None
                        } else {
                            Some(subscribed.get_heartbeat_interval_seconds())
                        };

                    self.scheduler.subscribed(&client,
                                              subscribed.get_framework_id(),
                                              heartbeat_interval_seconds)
                }
                Event_Type::OFFERS => {
                    let offers = event.get_offers();

                    // Split offers per-agent to save users the time of
                    // doing so.
                    for (_, offers) in offers.get_offers()
                                             .iter()
                                             .group_by(|o| o.get_agent_id()) {
                        self.scheduler.offers(&client, offers.to_vec());
                    }
                    for (_, inverse_offers) in offers.get_inverse_offers()
                                                     .iter()
                                                     .group_by(|o| {
                                                         o.get_agent_id()
                                                     }) {
                        self.scheduler
                            .inverse_offers(&client, inverse_offers.to_vec());
                    }

                }
                Event_Type::RESCIND =>
                    self.scheduler
                        .rescind(&client, event.get_rescind().get_offer_id()),
                Event_Type::UPDATE => {
                    let status = event.get_update().get_status();
                    self.scheduler.update(&client, status);
                    if self.conf.implicit_acknowledgements {
                        client.acknowledge(status.get_agent_id().clone(),
                                           status.get_task_id().clone(),
                                           status.get_uuid().to_vec());
                    }
                }
                Event_Type::MESSAGE => {
                    let message = event.get_message();
                    self.scheduler.message(&client,
                                           message.get_agent_id(),
                                           message.get_executor_id(),
                                           message.get_data().to_vec())
                }
                Event_Type::FAILURE => {
                    let failure = event.get_failure();
                    let agent_id = if !failure.has_agent_id() {
                        None
                    } else {
                        Some(failure.get_agent_id())
                    };
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
                    self.scheduler
                        .failure(&client, agent_id, executor_id, status)
                }
                Event_Type::ERROR => self.scheduler.error(&client,
                                                          event.get_error()
                                                               .get_message()
                                                               .to_string()),
                Event_Type::HEARTBEAT => self.scheduler.heartbeat(&client),
            }
        }
    }
}
