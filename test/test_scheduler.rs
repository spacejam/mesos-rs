use mesos::{Scheduler, SchedulerClient, run_protobuf_scheduler};
use mesos::proto::*;
use mesos::util::{task_info, launch};

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
    }

    fn offers(&mut self,
              client: &SchedulerClient,
              offers: Vec<&Offer>) {
        println!("received offers");

        let mut tasks = vec![];
        let mut offer_ids = vec![];
        for offer in offers {
            offer_ids.push(offer.get_id().clone());

            let name = format!("sleepy-{}", self.get_id());

            let mut task_id = TaskID::new();
            task_id.set_value(name.clone());

            let mut command = CommandInfo::new();
            command.set_value("env && sleep 10".to_string());

            let mut mem_scalar = Value_Scalar::new();
            mem_scalar.set_value(128f64);

            let mut mem = Resource::new();
            mem.set_name("mem".to_string());
            mem.set_role("*".to_string());
            mem.set_field_type(Value_Type::SCALAR);
            mem.set_scalar(mem_scalar);

            let mut cpus_scalar = Value_Scalar::new();
            cpus_scalar.set_value(1f64);

            let mut cpus = Resource::new();
            cpus.set_name("cpus".to_string());
            cpus.set_role("*".to_string());
            cpus.set_field_type(Value_Type::SCALAR);
            cpus.set_scalar(cpus_scalar);

            let resources = vec![mem, cpus];

            let task_info = task_info(name, &task_id, offer.get_agent_id(), &command, resources);

            tasks.push(task_info);
        }

        let mut operation = Operation::new();
        operation.set_field_type(Operation_Type::LAUNCH);
        operation.set_launch(launch(tasks));

        client.accept(offer_ids, vec![operation], None);
    }

    fn inverse_offers(&mut self,
              client: &SchedulerClient,
              inverse_offers: Vec<&InverseOffer>) {
        println!("received inverse offers");

        let mut inverse_offer_ids = vec![];
        for offer in inverse_offers {
            inverse_offer_ids.push(offer.get_id().clone());
        }

        client.decline(inverse_offer_ids, None).unwrap();
    }

    fn rescind(&mut self, client: &SchedulerClient, offer_id: &OfferID) {
        println!("received rescind");
    }

    fn update(&mut self, client: &SchedulerClient, status: &TaskStatus) {
        println!("received update");
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
}

#[test]
fn main() {
    let url = "http://localhost:5050".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;
    let framework_id = None;
    let mut scheduler = TestScheduler {
        max_id: 0,
    };

    run_protobuf_scheduler(url,
                           user,
                           name,
                           framework_timeout,
                           &mut scheduler,
                           framework_id);
}
