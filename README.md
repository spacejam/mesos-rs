# mesos-rs :globe_with_meridians:

Simple bindings for the Mesos v1 HTTP API.

```rust
use mesos::{Scheduler, SchedulerClient, run_protobuf_scheduler};
use mesos::proto::scheduler::*;
use mesos::proto::mesos::{Filters, Offer, OfferID, InverseOffer};

pub struct TestScheduler;

impl Scheduler for TestScheduler {
    fn subscribed(&mut self, client: &SchedulerClient, subscribed: &Event_Subscribed) {
        println!("received subscribed");
    }

    fn offers(&mut self, client: &SchedulerClient, offers: Vec<Offer>, inverse_offers: Vec<InverseOffer>) {
        println!("received offers");

        let mut offer_ids = vec![];
        for offer in offers {
            offer_ids.push(offer.get_id().clone());
        }
        for offer in inverse_offers {
            offer_ids.push(offer.get_id().clone());
        }

        let filters = Filters::new();

        client.decline(offer_ids, filters).unwrap();
    }

    fn rescind(&mut self, client: &SchedulerClient, rescind: &Event_Rescind) {
        println!("received rescind");
    }

    fn update(&mut self, client: &SchedulerClient, update: &Event_Update) {
        println!("received update");
    }

    fn message(&mut self, client: &SchedulerClient, message: &Event_Message) {
        println!("received message");
    }

    fn failure(&mut self, client: &SchedulerClient, failure: &Event_Failure) {
        println!("received failure");
    }

    fn error(&mut self, client: &SchedulerClient, error: &Event_Error) {
        println!("received error");
    }

    fn heartbeat(&mut self, client: &SchedulerClient) {
        println!("received heartbeat");
    }
}

#[test]
fn main() {
    let url = "http://localhost:5050/api/v1/scheduler".to_string();
    let user = "root".to_string();
    let name = "rust http".to_string();
    let framework_timeout = 0f64;
    let framework_id = None;

    let mut scheduler = TestScheduler;
    run_protobuf_scheduler(url, user, name, framework_timeout, &mut scheduler, framework_id);
}
```
