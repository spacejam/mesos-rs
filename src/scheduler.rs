use client::SchedulerClient;
use proto::scheduler::*;
use proto::mesos::{Offer, InverseOffer};

pub trait Scheduler {
    fn subscribed(&mut self, client: &SchedulerClient, subscribed: &Event_Subscribed);
    fn offers(&mut self, client: &SchedulerClient, offers: Vec<Offer>, inverse_offers: Vec<InverseOffer>);
    fn rescind(&mut self, client: &SchedulerClient, rescind: &Event_Rescind);
    fn update(&mut self, client: &SchedulerClient, update: &Event_Update);
    fn message(&mut self, client: &SchedulerClient, message: &Event_Message);
    fn failure(&mut self, client: &SchedulerClient, failure: &Event_Failure);
    fn error(&mut self, client: &SchedulerClient, error: &Event_Error);
    fn heartbeat(&mut self, client: &SchedulerClient) {}
}
