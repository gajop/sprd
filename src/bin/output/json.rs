use sprd::event::{Event, Print};

pub struct JsonOutput {}

impl JsonOutput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Print for JsonOutput {
    fn event(&self, event: Event) {
        let serialized = serde_json::to_string(&event).unwrap();
        println!("{serialized}");
    }
}
