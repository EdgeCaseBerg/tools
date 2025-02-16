use serde::{Deserialize, Serialize};

/// An action to be taken related to a sentiment, only supports showing an image for now
/// ```
/// let data = "{\"show\": \"./data/pic.png\" }";
/// let p: SentimentAction = serde_json::from_str(data);
/// assert(p.is_ok());
/// assert_eq(p.unwrap(), SentimentAction { show: "./data/pic.png" })
/// ```
#[derive(Debug, Serialize, Deserialize)]
struct SentimentAction {
	show: String
}

#[derive(Debug, Serialize, Deserialize)]
enum SentimentField {
	Positive,
	Negative,
	Neutral
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarityRange {
	low: f32,
	high: f32,
	field: SentimentField
}

#[derive(Debug, Serialize, Deserialize)]
enum Relation {
	GT,
	LT,
	EQ
}

#[derive(Debug, Serialize, Deserialize)]
struct PolarityRelation {
	relation: Relation,
	left: SentimentField,
	right: SentimentField
}

impl PolarityRelation {
	fn new(left: SentimentField, relation: Relation, right: SentimentField) -> Self {
		PolarityRelation {
			relation,
			left,
			right
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct SentimentCondition {
	contains_words: Option<Vec<String>>,
	polarity_ranges: Option<Vec<PolarityRange>>,
	polarity_relations: Option<Vec<PolarityRelation>>
}

impl SentimentCondition {
	fn is_empty(&self) -> bool {
		self.contains_words.is_none()  && 
		self.polarity_ranges.is_none() &&
		self.polarity_relations.is_none()
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct SentimentRule {
	priority: u32,
	action: SentimentAction,
	condition: SentimentCondition
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
		let range = PolarityRelation::new(SentimentField::Positive, Relation::LT, SentimentField::Negative);
		let result = serde_json::to_string(&range);
		assert!(result.is_ok());
		let string = result.unwrap();
		println!("{}", string);
		assert_eq!(string,  "{\"relation\":\"LT\",\"left\":\"Positive\",\"right\":\"Negative\"}");
	}
}