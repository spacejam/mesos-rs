use std::io::{self, Error, ErrorKind, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

use itertools::Itertools;

use executor_client::ExecutorClient;
use recordio::RecordIOCodec;
use proto::mesos::{FrameworkID, Offer};
use proto::executor::*;
use {Executor, ExecutorConf, util};

pub trait ExecutorRouter {
    fn run(&mut self,
           rx: Receiver<io::Result<Event>>,
           client: ExecutorClient,
           conf: ExecutorConf);
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

pub struct ExecutorCallbackRouter<'a> {
    pub executor: &'a mut Executor,
    pub conf: ExecutorConf,
}

impl <'a> ExecutorRouter for ProtobufCallbackRouter<'a> {
    fn run(&mut self,
           rx: Receiver<io::Result<Event>>,
           client: ExecutorClient,
           conf: ExecutorConf) {
        let mut state = State::Connected;
        for e in rx {
            if e.is_err() {
                if state.is_connected() {
                    state = State::Disconnected;
                    self.executor.disconnected();
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

                    self.executor.subscribed(&client,
                                              subscribed.get_framework_id(),
                                              heartbeat_interval_seconds)
                }
                Event_Type::LAUNCH => {
                    let offers = event.get_offers();

                    // Split offers per-agent to save users the time of
                    // doing so.
                    for (_, offers) in offers.get_offers()
                                             .iter()
                                             .group_by(|o| o.get_agent_id()) {
                        self.executor.offers(&client, offers.to_vec());
                    }
                    for (_, inverse_offers) in offers.get_inverse_offers()
                                                     .iter()
                                                     .group_by(|o| {
                                                         o.get_agent_id()
                                                     }) {
                        self.executor
                            .inverse_offers(&client, inverse_offers.to_vec());
                    }

                }
                Event_Type::MESSAGE => {
                    let message = event.get_message();
                    self.executor.message(&client,
                                           message.get_data().to_vec())
                }
                Event_Type::SHUTDOWN => self.executor.shutdown(&client),
                Event_Type::ERROR => self.executor.error(&client,
                                                          event.get_error()
                                                               .get_message()
                                                               .to_string()),
            }
        }
    }
}
