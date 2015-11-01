# mesos-rs :globe_with_meridians:

Simple bindings for the Mesos v1 HTTP API.

Roadmap:
- [x] scheduler
- [ ] executor
- [ ] zk master detection and failover

#### Running
```
[dependencies]
mesos = "0.2.6"
```

###### Scheduler

```rust
extern crate mesos;

use self::mesos::{Scheduler, SchedulerClient, run_protobuf_scheduler};
use self::mesos::proto::*;
use self::mesos::util;

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
        // or use this if you don't require special filtering
        let mut offer_mem = util::get_scalar_resource_sum("mem", offers);

        let mut tasks = vec![];
        while offer_cpus >= 1f64 && offer_mem >= 128f64 {
            let name = format!("sleepy-{}", self.get_id());

            let mut task_id = TaskID::new();
            task_id.set_value(name.clone());

            let mut command = CommandInfo::new();
            command.set_value("env && sleep 10".to_string());

            let mem = util::scalar("mem", "*", 128f64);
            let cpus = util::scalar("cpus", "*", 1f64);

            let resources = vec![mem, cpus];

            let task_info = util::task_info(name,
                                            &task_id,
                                            agent_id,
                                            &command,
                                            resources);
            tasks.push(task_info);
            offer_cpus -= 1f64;
            offer_mem -= 128f64;
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

fn main() {
    let url = "http://localhost:5050".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;
    let framework_id = None;
    let mut scheduler = TestScheduler { max_id: 0 };

    run_protobuf_scheduler(url,
                           user,
                           name,
                           framework_timeout,
                           &mut scheduler,
                           framework_id);
}
```
