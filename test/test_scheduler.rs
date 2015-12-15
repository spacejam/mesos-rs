extern crate rand;
extern crate protobuf;

use std::thread;

use mesos::{Scheduler, SchedulerClient, SchedulerConf,
            ProtobufCallbackRouter, run_protobuf_scheduler};
use mesos::proto::*;
use mesos::util;

struct TestScheduler {
    max_id: u64,
}

impl TestScheduler {
    fn get_id(&mut self) -> u64 {
        self.max_id += 1;
        self.max_id
    }
}

impl Scheduler for TestScheduler {
    fn subscribed(&mut self,
                  client: &SchedulerClient,
                  framework_id: &FrameworkID,
                  heartbeat_interval_seconds: Option<f64>) {
        println!("received subscribed");

        client.reconcile(vec![]);
    }

    // Inverse offers are only available with the HTTP API
    // and are great for doing things like triggering
    // replication with stateful services before the agent
    // goes down for maintenance.
    fn inverse_offers(&mut self,
                      client: &SchedulerClient,
                      inverse_offers: Vec<&InverseOffer>) {
        println!("received inverse offers");

        // this never lets go willingly
        let offer_ids = inverse_offers.iter()
                                      .map(|o| o.get_id().clone())
                                      .collect();
        client.decline(offer_ids, None);
    }

    fn offers(&mut self, client: &SchedulerClient, offers: Vec<&Offer>) {

        // Offers are guaranteed to be for the same agent, and
        // there will be at least one.
        let agent_id = offers[0].get_agent_id();

        println!("received {} offers from agent {}",
                 offers.len(),
                 agent_id.get_value());

        let offer_ids: Vec<OfferID> = offers.iter()
                                            .map(|o| o.get_id().clone())
                                            .collect();

        // get resources with whatever filters you need
        let mut offer_cpus: f64 = offers.iter()
                                        .flat_map(|o| o.get_resources())
                                        .filter(|r| r.get_name() == "cpus")
                                        .map(|c| c.get_scalar())
                                        .fold(0f64, |acc, cpu_res| {
                                            acc + cpu_res.get_value()
                                        });

        let mut offer_ports: Vec<(u64, u64)> = offers.iter()
                                        .flat_map(|o| o.get_resources())
                                        .filter(|r| r.get_name() == "ports")
                                        .map(|p| p.get_ranges())
                                        .fold(vec![], |mut acc, ports_res| {
                                            for range in ports_res.get_range() {
                                                acc.push((range.get_begin(), range.get_end()));
                                            }
                                            acc
                                        });

        // or use this if you don't require special filtering
        let mut offer_mem = util::get_scalar_resource_sum("mem", offers);

        let mut tasks = vec![];
        while offer_cpus >= 1f64 && offer_mem >= 128f64 && offer_ports.len() != 0 {
            let name = &*format!("sleepy-{}", self.get_id());

            let task_id = util::task_id(name);

            let mut command = CommandInfo::new();
            command.set_value("env && while true; do echo yo $PORT0 | nc -vl $PORT0; done".to_string());

            let mut label = Label::new();
            if rand::random::<bool>() {
                label.set_key("vip_PORT0A".to_string());
            } else if rand::random::<bool>() {
                label.set_key("vip_PORT2".to_string());
            } else {
                label.set_key("vip_PORT0".to_string());
            }
            label.set_value("tcp://1.2.3.4:5".to_string());

            let mut labels = Labels::new();
            labels.set_labels(protobuf::RepeatedField::from_vec(vec![label]));

            let mem = util::scalar("mem", "*", 128f64);
            let cpus = util::scalar("cpus", "*", 1f64);

            let mut port_ranges = vec![];

            for i in 0..2 {
                if offer_ports.len() == 0 {
                    break;
                }
                let port_range = offer_ports.pop().unwrap();
                if port_range.0 + 2 <= port_range.1 {
                    offer_ports.push((port_range.0 + 2, port_range.1));
                }
                port_ranges.push((port_range.0, port_range.0));

                let mut env_var = Environment_Variable::new();
                env_var.set_name("PORT0".to_string());
                env_var.set_value(format!("{}", port_range.0));

                let mut env = Environment::new();
                env.set_variables(protobuf::RepeatedField::from_vec(vec![env_var]));
                command.set_environment(env);
            }
            println!("{:?}", port_ranges);

            let ports = util::range("ports", "*", port_ranges);

            let resources = vec![mem, cpus, ports];

            let mut task_info = util::task_info(name,
                                            &task_id,
                                            agent_id,
                                            &command,
                                            resources);

            task_info.set_labels(labels);

            let mut health_command = CommandInfo::new();
            health_command.set_value("false".to_string());

            let mut health_check = HealthCheck::new();
            health_check.set_command(health_command);
            health_check.set_delay_seconds(0f64);
            health_check.set_interval_seconds(1f64);
            health_check.set_timeout_seconds(1f64);
            health_check.set_consecutive_failures(10);
            health_check.set_grace_period_seconds(1f64);

            task_info.set_health_check(health_check);

            tasks.push(task_info);
            offer_cpus -= 1f64;
            offer_mem -= 128f64;
            thread::sleep_ms(1000);
        }
        client.launch(offer_ids, tasks, None);
    }

    fn rescind(&mut self, client: &SchedulerClient, offer_id: &OfferID) {
        println!("received rescind");
    }

    fn update(&mut self, client: &SchedulerClient, status: &TaskStatus) {
        println!("received update {:?} from {}",
                 status.get_state(),
                 status.get_task_id().get_value());
    }

    fn message(&mut self,
               client: &SchedulerClient,
               agent_id: &AgentID,
               executor_id: &ExecutorID,
               data: Vec<u8>) {
        println!("received message");
    }

    fn failure(&mut self,
               client: &SchedulerClient,
               agent_id: Option<&AgentID>,
               executor_id: Option<&ExecutorID>,
               status: Option<i32>) {
        println!("received failure");
    }

    fn error(&mut self, client: &SchedulerClient, message: String) {
        println!("received error");
    }

    fn heartbeat(&mut self, client: &SchedulerClient) {
        println!("received heartbeat");
    }

    fn disconnected(&mut self) {
        println!("disconnected from scheduler");
    }
}

#[test]
fn main() {
    let mut scheduler = TestScheduler { max_id: 0 };

    let conf = SchedulerConf {
        master_url: "http://localhost:5050".to_string(),
        user: "root".to_string(),
        name: "rust http".to_string(),
        framework_timeout: 0f64,
        implicit_acknowledgements: true,
        framework_id: None,
    };

    // If you don't like the callback approach, you can implement
    // an event router of your own.  This is merely provided for
    // those familiar with the mesos libraries in other languages.
    let mut router = ProtobufCallbackRouter {
        scheduler: &mut scheduler,
        conf: conf.clone(),
    };

    run_protobuf_scheduler(&mut router, conf)
}
