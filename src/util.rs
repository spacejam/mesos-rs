use hyper::header::{Accept, Connection, ContentType, Headers, Quality,
                    QualityItem, qitem};
use hyper::mime::{Mime, SubLevel, TopLevel};
use protobuf::{self, Message};

use proto::mesos::*;

pub fn protobuf_headers() -> Headers {
    let mut headers = Headers::new();

    headers.set(Accept(vec![
            qitem(Mime(TopLevel::Text, SubLevel::Html, vec![])),
            qitem(Mime(TopLevel::Application,
            SubLevel::Ext("x-protobuf".to_owned()), vec![])),
        ]));

    headers.set(ContentType(Mime(TopLevel::Application,
                                 SubLevel::Ext("x-protobuf".to_owned()),
                                 vec![])));

    headers
}

pub fn framework_info(user: String,
                      name: String,
                      failover_timeout: f64)
                      -> FrameworkInfo {
    let mut framework_info = FrameworkInfo::new();
    framework_info.set_user(user);
    framework_info.set_name(name);
    framework_info.set_failover_timeout(failover_timeout);
    framework_info
}

pub fn task_info(name: String,
                 task_id: &TaskID,
                 agent_id: &AgentID,
                 command: &CommandInfo,
                 resources: Vec<Resource>)
                 -> TaskInfo {

    let mut task_info = TaskInfo::new();
    task_info.set_name(name);
    task_info.set_task_id(task_id.clone());
    task_info.set_agent_id(agent_id.clone());
    task_info.set_command(command.clone());
    task_info.set_resources(protobuf::RepeatedField::from_vec(resources));
    task_info
}

pub fn launch_operation(task_infos: Vec<TaskInfo>) -> Operation {
    let mut launch = Operation_Launch::new();
    launch.set_task_infos(protobuf::RepeatedField::from_vec(task_infos));

    let mut operation = Operation::new();
    operation.set_field_type(Operation_Type::LAUNCH);
    operation.set_launch(launch);
    operation
}

pub fn scalar(name: &'static str, role: &'static str, value: f64) -> Resource {
    let mut scalar = Value_Scalar::new();
    scalar.set_value(value);

    let mut res = Resource::new();
    res.set_name(name.to_string());
    res.set_role(role.to_string());
    res.set_field_type(Value_Type::SCALAR);
    res.set_scalar(scalar);
    res
}

pub fn get_scalar_resource_sum(name: &'static str, offers: Vec<&Offer>) -> f64 {
    offers.iter()
          .flat_map(|o| o.get_resources())
          .filter(|r| r.get_name() == "mem")
          .map(|c| c.get_scalar())
          .fold(0f64, |acc, mem_res| acc + mem_res.get_value())
}
