pub struct ProgressBarInfo {
    pub pre_message: String,
    pub post_message: String,
}

impl ProgressBarInfo {
    pub fn new(pre_message: &str, post_message: &str) -> Self {
        Self {
            pre_message: pre_message.to_string(),
            post_message: post_message.to_string(),
        }
    }
}
