use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// An action to be taken related to a sentiment, only supports showing an image for now
/// ```
/// let data = "{\"show\": \"./data/pic.png\" }";
/// let p: SentimentAction = serde_json::from_str(data);
/// assert(p.is_ok());
/// assert_eq(p.unwrap(), SentimentAction { show: "./data/pic.png" })
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SentimentAction {
	pub show: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentimentField {
	Positive,
	Negative,
	Neutral
}

pub struct ContextPolarity {
    pub positive: f64,
    pub negative: f64,
    pub neutral: f64
}

impl ContextPolarity {
    fn for_field(&self, field: &SentimentField) -> f64 {
         match field {
            SentimentField::Positive => self.positive,
            SentimentField::Negative => self.negative,
            SentimentField::Neutral => self.neutral ,
        }
    }
}

/// Expresses a condition that the given sentiment field will be within the range (inclusive) 
#[derive(Debug, Serialize, Deserialize)]
pub struct PolarityRange {
	pub low: f64,
	pub high: f64,
	pub field: SentimentField
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Relation {
	/// Greater than
	GT,
	/// Less than
	LT,
	/// Equal to
	EQ
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolarityRelation {
	pub relation: Relation,
	pub left: SentimentField,
	pub right: SentimentField
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentCondition {
	pub contains_words: Option<Vec<String>>,
	pub polarity_ranges: Option<Vec<PolarityRange>>,
	pub polarity_relations: Option<Vec<PolarityRelation>>
}

impl SentimentCondition {
	pub fn is_empty(&self) -> bool {
		self.contains_words.is_none()  && 
		self.polarity_ranges.is_none() &&
		self.polarity_relations.is_none()
	}

	/// Returns None if no rule defined, Some(T|F) for if there was a match otherwise
	fn context_contains_words(&self, sentence: &str) -> Option<bool> {
	    if let Some(words) = &self.contains_words {
	        let contains_words = words.iter().any(|word| {
	            sentence.contains(word)
	        });
	        return Some(contains_words);
	    }
	    None
	}

	fn context_in_polarity_range(&self, polarity: &ContextPolarity) -> Option<bool> {
	    if let Some(ranges) = &self.polarity_ranges {
	        let is_in_range = ranges.iter().all(|range| {
	            let field = polarity.for_field(&range.field);
	            range.low <= field && field <= range.high
	        });
	        return Some(is_in_range)
	    }
	    None
	}

	fn context_has_polarity_relations(&self, polarity: &ContextPolarity) -> Option<bool> {
	    if let Some(relations) = &self.polarity_relations {
	        let relation_is_true = relations.iter().all(|relation| {
	            let left = polarity.for_field(&relation.left);
	            let right = polarity.for_field(&relation.right);
	            match &relation.relation {
	                Relation::GT => left > right,
	                Relation::LT => left < right,
	                Relation::EQ => left == right,
	            }
	        });
	        return Some(relation_is_true);
	    }
	    None
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentRule {
	pub priority: u32,
	pub action: SentimentAction,
	pub condition: SentimentCondition
}

impl SentimentRule {
	pub fn applies_to(&self, sentence: &str, polarity: &ContextPolarity) -> bool {
		let rule_checks = [
            self.condition.context_contains_words(sentence),
            self.condition.context_in_polarity_range(polarity),
            self.condition.context_has_polarity_relations(polarity)
        ];

        let applicable_checks: Vec<bool> = rule_checks
            .iter()
            .filter_map(|&rule_result| rule_result)
            .collect();

        if applicable_checks.is_empty() {
            return false;
        }

        applicable_checks.iter().all(|&bool| bool)
	}
}

pub fn load_from_file(path: &PathBuf) -> Result<Vec<SentimentRule>, Box<dyn Error>> {
	let file = File::open(path)?;
	let reader = BufReader::new(file);
	let parsed: Vec<SentimentRule> = serde_json::from_reader(reader)?;
	let mut valid_rules: Vec<SentimentRule> = parsed.into_iter().filter(|unvalidated_rule| {
		!unvalidated_rule.condition.is_empty()
	}).collect();
	valid_rules.sort_by_key(|rule| rule.priority);
	valid_rules.reverse();
	Ok(valid_rules)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sentiment_condition_is_empty_works() {
		let s = "{}";
		let j: serde_json::Result<SentimentCondition> = serde_json::from_str(s);
		assert!(j.is_ok());
		assert!(j.unwrap().is_empty());
	}

	#[test]
	fn polarity_ranges_serialize_as_expected() {
		let range = PolarityRelation {
			left: SentimentField::Positive, 
			relation: Relation::LT,
			right: SentimentField::Negative
		};
		let result = serde_json::to_string(&range);
		assert!(result.is_ok());
		let string = result.unwrap();
		assert_eq!(string,  "{\"relation\":\"LT\",\"left\":\"Positive\",\"right\":\"Negative\"}");
	}

	#[test]
	fn rule_is_only_true_if_all_conditions_match() {
		let rule = SentimentRule {
			priority: 0,
			action: SentimentAction {
				show: String::new()
			},
			condition: SentimentCondition {
				contains_words: Some(vec![
					"word".to_string(), 
					"yay".to_string()
				]),
				polarity_ranges: Some(vec![
					PolarityRange {
						low: 0.5,
						high: 0.75,
						field: SentimentField::Positive					
					},
					PolarityRange {
						low: 0.0,
						high: 0.25,
						field: SentimentField::Negative					
					}
				]),
				polarity_relations: None
			}
		};

		let polarity1 = ContextPolarity {
			positive: 0.0,
			negative: 0.9,
			neutral: 0.0,
		};

		let polarity2 = ContextPolarity {
			positive: 0.6,
			negative: 0.25,
			neutral: 0.0,
		};

		let should_be_false = rule.applies_to("no match here", &polarity1);
		assert_eq!(should_be_false, false);

		let should_be_false = rule.applies_to("no match here", &polarity2);
		assert_eq!(should_be_false, false);
		
		let should_be_true = rule.applies_to("word", &polarity2);
		assert_eq!(should_be_true, true);
	}
}