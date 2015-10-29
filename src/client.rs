use std::io;

use hyper;
use hyper::Client;
use hyper::client::response::Response;
use hyper::header::{Connection, ContentType, Headers, Accept, QualityItem, Quality, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use protobuf::Message;

use proto::scheduler::{Call, Call_Type, Call_Subscribe};
use proto::mesos::{FrameworkInfo};

pub fn subscribe(url: String, user: String, name: String, failover_timeout: f64) -> hyper::Result<Response> {
    let mut subscribe = Call_Subscribe::new();
    subscribe.set_framework_info(
        framework_info(user, name, failover_timeout)
    );
    
    let mut call = Call::new();
    call.set_subscribe(subscribe);
    call.set_field_type(Call_Type::SUBSCRIBE);

    post(url, &*call.write_to_bytes().unwrap())
}

fn framework_info(user: String, name: String, failover_timeout: f64) -> FrameworkInfo {
    let mut framework_info = FrameworkInfo::new();
    framework_info.set_user(user);
    framework_info.set_name(name);
    framework_info.set_failover_timeout(failover_timeout);
    framework_info   
}

fn post(url: String, data: &[u8]) -> hyper::Result<Response> {
    let client = Client::new();

    let mut headers = Headers::new();

    headers.set(
        Accept(vec![
            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
            qitem(Mime(TopLevel::Application,
            SubLevel::Ext("x-protobuf".to_owned()), vec![])),
        ])
    );

    headers.set(
        ContentType(Mime(TopLevel::Application,
            SubLevel::Ext("x-protobuf".to_owned()), vec![])),
    );

    client.post(&*url)
        .headers(headers)
        .body(data)
        .send()
}

