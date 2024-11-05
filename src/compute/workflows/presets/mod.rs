use dkn_workflows::Workflow;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref GENERATION_WORKFLOW: Workflow = {
        serde_json::from_str(include_str!("generation.json"))
            .expect("could not parse generation workflow")
    };
}
