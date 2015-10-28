use std::io;

use hyper::Client;
use hyper::client::response::Response;
use hyper::header::{Connection, ContentType, Headers, Accept, QualityItem, Quality, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use protobuf::Message;

use proto::scheduler::{Call, Call_Type, Call_Subscribe};
use proto::mesos::{FrameworkInfo};


#[test]
fn main() {
    let url = "http://localhost:5050/api/v1/scheduler".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;

    let mut res = subscribe(url, user, name, framework_timeout);

    println!("Response: {}", res.status);
    println!("Headers:\n{}", res.headers);
    io::copy(&mut res, &mut io::stdout()).unwrap();
}

pub fn subscribe(url: String, user: String, name: String, failover_timeout: f64) -> Response {
    let mut subscribe = Call_Subscribe::new();
    subscribe.set_framework_info(
        framework_info(user, name, failover_timeout)
    );
    
    let mut call = Call::new();
    call.set_subscribe(subscribe);
    call.set_field_type(Call_Type::SUBSCRIBE);

    let url = "http://localhost:5050/api/v1/scheduler";

    post(url, &*call.write_to_bytes().unwrap())
}

fn framework_info(user: String, name: String, failover_timeout: f64) -> FrameworkInfo {
    let mut framework_info = FrameworkInfo::new();
    framework_info.set_user(user);
    framework_info.set_name(name);
    framework_info.set_failover_timeout(failover_timeout);
    framework_info   
}

fn post(url: &str, data: &[u8]) -> Response {
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
        .send().unwrap()
}
