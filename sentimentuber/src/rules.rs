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
#[derive(Debug, Serialize, Deserialize)]
struct SentimentAction {
	pub show: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentimentField {
	Positive,
	Negative,
	Neutral
}

/// Expresses a condition that the given sentiment field will be within the range (inclusive) 
#[derive(Debug, Serialize, Deserialize)]
struct PolarityRange {
	pub low: f32,
	pub high: f32,
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
	relation: Relation,
	left: SentimentField,
	right: SentimentField
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentCondition {
	contains_words: Option<Vec<String>>,
	polarity_ranges: Option<Vec<PolarityRange>>,
	polarity_relations: Option<Vec<PolarityRelation>>
}

impl SentimentCondition {
	pub fn is_empty(&self) -> bool {
		self.contains_words.is_none()  && 
		self.polarity_ranges.is_none() &&
		self.polarity_relations.is_none()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentRule {
	priority: u32,
	action: SentimentAction,
	condition: SentimentCondition
}

pub fn load_from_file(path: &PathBuf) -> Result<Vec<SentimentRule>, Box<dyn Error>> {
	let file = File::open(path)?;
	let reader = BufReader::new(file);
	let parsed: Vec<SentimentRule> = serde_json::from_reader(reader)?;
	let valid_rules = parsed.into_iter().filter(|unvalidated_rule| {
		!unvalidated_rule.condition.is_empty()
	}).collect();
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
		println!("{}", string);
		assert_eq!(string,  "{\"relation\":\"LT\",\"left\":\"Positive\",\"right\":\"Negative\"}");
	}
}