use std::io;
use std::sync::{Arc, Mutex};

use hyper;
use hyper::Client;
use hyper::client::response::Response;
use hyper::header::{Accept, Connection, ContentType, Headers, Quality,
                    QualityItem, qitem};
use protobuf::{self, Message};

use proto::executor::{Call, Call_Message, Call_Subscribe, Call_Type,
                      Call_Update};
use proto::mesos::{AgentID, ExecutorID, Filters, FrameworkID, FrameworkInfo,
                   OfferID, Operation, Request, TaskID, TaskInfo, TaskStatus};
use util;

#[derive(Clone)]
pub struct ExecutorClient {
    pub url: String,
    pub framework_id: Arc<Mutex<Option<FrameworkID>>>,
    pub executor_id: Arc<Mutex<Option<ExecutorID>>>,
}

impl ExecutorClient {
    fn subscribe(&self,
                 tasks: Vec<TaskInfo>,
                 updates: Vec<Call_Update>)
                 -> hyper::Result<Response> {
        let mut subscribe = Call_Subscribe::new();
        subscribe.set_tasks(protobuf::RepeatedField::from_vec(tasks));
        subscribe.set_updates(protobuf::RepeatedField::from_vec(updates));

        let mut call = Call::new();
        call.set_field_type(Call_Type::SUBSCRIBE);
        call.set_subscribe(subscribe);

        self.post(&mut call)
    }

    fn update(&self,
              status: TaskStatus,
              timestamp: f64,
              uuid: Vec<u8>)
              -> hyper::Result<Response> {
        let mut update = Call_Update::new();
        update.set_status(status);
        update.set_timestamp(timestamp);
        update.set_uuid(uuid);

        let mut call = Call::new();
        call.set_field_type(Call_Type::SUBSCRIBE);
        call.set_update(update);

        self.post(&mut call)
    }

    fn message(&self, data: Vec<u8>) -> hyper::Result<Response> {
        let mut message = Call_Message::new();
        message.set_data(data);

        let mut call = Call::new();
        call.set_field_type(Call_Type::SUBSCRIBE);
        call.set_message(message);

        self.post(&mut call)
    }

    fn post(&self, call: &mut Call) -> hyper::Result<Response> {
        {
            let framework_id = self.framework_id.lock().unwrap();
            if framework_id.is_some() {
                let framework_id_clone = framework_id.clone();
                call.set_framework_id(framework_id_clone.unwrap());
            }
        }
        {
            let executor_id = self.executor_id.lock().unwrap();
            if executor_id.is_some() {
                let executor_id_clone = executor_id.clone();
                call.set_executor_id(executor_id_clone.unwrap());
            }
        }

        let client = Client::new();

        let data = &*call.write_to_bytes().unwrap();

        client.post(&*self.url)
              .headers(util::protobuf_headers())
              .body(data)
              .send()
    }
}
