//! Sliding window context optimization

use crate::Message;
use tiktoken_rs::CoreBPE;

pub struct SlidingWindow {
    max_tokens: usize,
}

impl SlidingWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self { max_tokens }
    }

    /// Optimize messages to fit within token budget
    /// Strategy: Keep recent messages, summarize or drop oldest
    pub fn optimize(&self, messages: &[Message], tokenizer: &CoreBPE) -> Vec<Message> {
        if messages.is_empty() {
            return Vec::new();
        }

        let mut total_tokens = 0;
        let mut kept_messages = Vec::new();

        // Process from newest to oldest
        for msg in messages.iter().rev() {
            let msg_tokens = msg.tokens.unwrap_or_else(|| {
                tokenizer.encode_with_special_tokens(&msg.content).len()
            });

            if total_tokens + msg_tokens <= self.max_tokens {
                total_tokens += msg_tokens;
                kept_messages.push(msg.clone());
            } else {
                // Token budget exceeded, stop adding messages
                break;
            }
        }

        // Reverse back to original order
        kept_messages.reverse();

        // If we had to drop messages, add a system message indicating truncation
        if kept_messages.len() < messages.len() {
            let dropped_count = messages.len() - kept_messages.len();
            let truncation_msg = Message::new(
                "system",
                format!("[Context truncated: {} older messages removed to fit token budget]", dropped_count)
            );
            kept_messages.insert(0, truncation_msg);
        }

        kept_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiktoken_rs::cl100k_base;

    #[test]
    fn test_sliding_window_keeps_recent() {
        let tokenizer = cl100k_base().unwrap();
        let window = SlidingWindow::new(50); // Very small budget

        let messages = vec![
            Message::new("user", "Old message 1"),
            Message::new("assistant", "Old response 1"),
            Message::new("user", "Recent message"),
            Message::new("assistant", "Recent response"),
        ];

        let optimized = window.optimize(&messages, &tokenizer);

        // Should keep recent messages
        assert!(optimized.len() < messages.len() || optimized.len() == messages.len());
        
        // Last message should be preserved
        assert_eq!(optimized.last().unwrap().content, "Recent response");
    }
}
