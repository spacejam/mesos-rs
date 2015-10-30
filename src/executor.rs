use executor_client::ExecutorClient;
use proto::*;

pub trait Executor {
    fn subscribed(&self,
                  client: &ExecutorClient,
                  executor_info: &ExecutorInfo,
                  framework_info: &FrameworkInfo,
                  agent_info: &AgentInfo);
    fn launch(&self, client: &ExecutorClient, task_info: &TaskInfo);
    fn kill(&self, client: &ExecutorClient, task_id: &TaskID);
    fn acknowledged(&self,
                    client: &ExecutorClient,
                    task_id: &TaskID,
                    uuid: Vec<u8>);
    fn message(&self, client: &ExecutorClient, data: Vec<u8>);
    fn shutdown(&self, client: &ExecutorClient);
    fn error(&self, client: &ExecutorClient, message: String);
}
