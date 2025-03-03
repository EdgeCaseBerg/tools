use std::time::Instant;
use std::time::Duration;
use std::collections::VecDeque;

use crate::rules::ContextPolarity;
use crate::rules::SentimentAction;
use crate::rules::SentimentRule;

pub struct SentimentEngine<PolarityClosure>
where
    PolarityClosure: Fn(&str) -> ContextPolarity
{
    text_context: VecDeque<(Instant, String)>,
    current_context: String,
    rules: Vec<SentimentRule>,
    polarity_closure: PolarityClosure,
    context_retention_seconds: u64,
}

impl<PolarityClosure> SentimentEngine<PolarityClosure>
where
    PolarityClosure: Fn(&str) -> ContextPolarity
{
    pub fn new(polarity_closure: PolarityClosure) -> Self {
        SentimentEngine {
            text_context: VecDeque::new(),
            current_context: String::new(),
            rules: Vec::new(),
            polarity_closure,
            context_retention_seconds: 10
        }
    }

    pub fn add_context(&mut self, new_content: String) {
        let right_now = Instant::now();
        let drop_time = right_now - Duration::from_secs(self.context_retention_seconds);
        let mut current_context = String::new();
        self.text_context.push_back((right_now, new_content));
        self.text_context.retain(|tuple| {
             if tuple.0.ge(&drop_time) {
                 current_context.push_str(&tuple.1.clone());
             }
             tuple.0.ge(&drop_time)
        });
        self.current_context = current_context.to_lowercase();
    }

    pub fn set_rules(&mut self, rules: Vec<SentimentRule>) {
        self.rules = rules;
    }

    pub fn set_context_duration(&mut self, seconds: u64) {
    	self.context_retention_seconds = seconds;
    }

    fn get_polarity(&self) -> ContextPolarity {
        (self.polarity_closure)(&self.current_context)
    }

    pub fn get_action(&self) -> SentimentAction {
        let polarity = self.get_polarity();
        let maybe_action = self.rules.iter().find(|&rule| {
            rule.applies_to(&self.current_context, &polarity)
        });
        match maybe_action {
            Some(rule_based_action) => rule_based_action.action.clone(),
            None => SentimentAction {
                show: "./data/neutral.png".to_string()
            }
        }
    }
}
