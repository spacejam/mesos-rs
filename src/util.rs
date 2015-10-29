use hyper::header::{Connection, ContentType, Headers, Accept, QualityItem, Quality, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};

use proto::mesos::{FrameworkInfo, OfferID, Filters};

pub fn protobuf_headers() -> Headers {
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

    headers
}

pub fn framework_info(user: String, name: String, failover_timeout: f64) -> FrameworkInfo {
    let mut framework_info = FrameworkInfo::new();
    framework_info.set_user(user);
    framework_info.set_name(name);
    framework_info.set_failover_timeout(failover_timeout);
    framework_info   
}
