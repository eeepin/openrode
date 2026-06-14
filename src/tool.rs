#[derive(Serialize, Deserialize, Debug)]
pub enum ToolName {
    Read,
    Write,
    Bash,
}

impl AgentTool {
    pub fn read() -> Self {
        serde_json::from_str(include_str!("../tools/read.json")).unwrap()
    }

    pub fn write() -> Self {
        serde_json::from_str(include_str!("../tools/write.json")).unwrap()
    }

    pub fn bash() -> Self {
        serde_json::from_str(include_str!("../tools/bash.json")).unwrap()
    }
}
