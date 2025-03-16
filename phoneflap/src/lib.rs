use bstr::ByteSlice;
use std::fs;
use std::{borrow::Cow, collections::HashMap};

pub fn parse_dictionary(dictionary_file_path: &str) -> HashMap<String, PhonemeSet> {
    let not_utf8: Vec<u8> = fs::read(dictionary_file_path).expect("Could not load dictionary file");
    let definitions: HashMap<String, PhonemeSet> = not_utf8
        .lines()
        .map(decode)
        .filter(|line| !line.starts_with(";;;"))
        .filter_map(|line| PhonemeSet::from(&line))
        .map(|p| (p.word.clone(), p))
        .collect();
    definitions
}

fn decode(line: &[u8]) -> Cow<'_, str> {
    if let Ok(s) = std::str::from_utf8(line) {
        Cow::Borrowed(s)
    } else {
        // naive ISO-8859-1 implementation that does not handle Windows-1252
        Cow::Owned(line.iter().map(|b| *b as char).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhonemeSet {
    pub set: Vec<Phoneme>,
    pub word: String,
}

impl PhonemeSet {
    fn from(raw_line: &str) -> Option<Self> {
        if !raw_line.contains("  ") {
            return None;
        }
        let split_point = raw_line.find("  ").unwrap();
        let (word, phones_and_stresses) = raw_line.split_at(split_point);

        let set = PhonemeSet::parse_phoneme(phones_and_stresses);

        Some(PhonemeSet {
            word: word.to_string(),
            set,
        })
    }

    fn parse_phoneme(raw_line_data: &str) -> Vec<Phoneme> {
        raw_line_data.split(" ").filter_map(Phoneme::from).collect()
    }

    pub fn vowel_count(&self) -> usize {
        self.set
            .iter()
            .filter(|phoneme| phoneme.phone.contains_vowel())
            .count()
    }


}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Phoneme {
    pub phone: Phone,
    pub stress: LexicalStress,
}

impl Phoneme {
    /// expects AA or AA0, AA1, AA2 where AA is any phoneme of the 39.
    fn from(phone_and_stress: &str) -> Option<Self> {
        let stress = match phone_and_stress.chars().find(|&c| c.is_ascii_digit()) {
            None => LexicalStress::NoStress,
            Some(stress_char) => match stress_char {
                '1' => LexicalStress::Primary,
                '2' => LexicalStress::Secondary,
                _ => LexicalStress::NoStress,
            },
        };

        let raw_phone: String = phone_and_stress
            .chars()
            .take_while(|c| !c.is_ascii_digit())
            .collect();

        Phone::from(&raw_phone).map(|phone| Self { phone, stress })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Phone {
    AA,
    AE,
    AH,
    AO,
    AW,
    AY,
    B,
    CH,
    D,
    DH,
    EH,
    ER,
    EY,
    F,
    G,
    HH,
    IH,
    IY,
    JH,
    K,
    L,
    M,
    N,
    NG,
    OW,
    OY,
    P,
    R,
    S,
    SH,
    T,
    TH,
    UH,
    UW,
    V,
    W,
    Y,
    Z,
    ZH,
}

impl Phone {
    pub fn contains_vowel(&self) -> bool {
        match self {
            Self::AA => true,
            Self::AE => true,
            Self::AH => true,
            Self::AO => true,
            Self::AW => true,
            Self::AY => true,
            Self::EH => true,
            Self::ER => true,
            Self::EY => true,
            Self::IH => true,
            Self::IY => true,
            Self::OW => true,
            Self::OY => true,
            Self::UH => true,
            Self::UW => true,
            _ => false,
        }
    }

    pub fn from(raw: &str) -> Option<Phone> {
        match raw {
            "AA" => Some(Phone::AA),
            "AE" => Some(Phone::AE),
            "AH" => Some(Phone::AH),
            "AO" => Some(Phone::AO),
            "AW" => Some(Phone::AW),
            "AY" => Some(Phone::AY),
            "B" => Some(Phone::B),
            "CH" => Some(Phone::CH),
            "D" => Some(Phone::D),
            "DH" => Some(Phone::DH),
            "EH" => Some(Phone::EH),
            "ER" => Some(Phone::ER),
            "EY" => Some(Phone::EY),
            "F" => Some(Phone::F),
            "G" => Some(Phone::G),
            "HH" => Some(Phone::HH),
            "IH" => Some(Phone::IH),
            "IY" => Some(Phone::IY),
            "JH" => Some(Phone::JH),
            "K" => Some(Phone::K),
            "L" => Some(Phone::L),
            "M" => Some(Phone::M),
            "N" => Some(Phone::N),
            "NG" => Some(Phone::NG),
            "OW" => Some(Phone::OW),
            "OY" => Some(Phone::OY),
            "P" => Some(Phone::P),
            "R" => Some(Phone::R),
            "S" => Some(Phone::S),
            "SH" => Some(Phone::SH),
            "T" => Some(Phone::T),
            "TH" => Some(Phone::TH),
            "UH" => Some(Phone::UH),
            "UW" => Some(Phone::UW),
            "V" => Some(Phone::V),
            "W" => Some(Phone::W),
            "Y" => Some(Phone::Y),
            "Z" => Some(Phone::Z),
            "ZH" => Some(Phone::ZH),
            "" => None,
            _ => {
                eprintln!("no match on {:?}", raw);
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, Copy)]
pub enum LexicalStress {
    NoStress, // kind of want to use None but don't want to get confused!
    Primary,
    Secondary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_parses_properly() {
        let maybe_aa = Phone::from("AA");
        assert!(maybe_aa.is_some());
        assert_eq!(Phone::AA, maybe_aa.unwrap());
    }

    #[test]
    fn phone_returns_none_for_bad_input() {
        let maybe_aa = Phone::from("");
        assert!(maybe_aa.is_none());
    }

    #[test]
    fn phoneme_parses_4_types_of_valid_input() {
        let cases = vec![
            Phoneme::from("AA"),
            Phoneme::from("AA0"),
            Phoneme::from("AA1"),
            Phoneme::from("AA2"),
        ];

        for case in &cases {
            assert!(case.is_some());
            assert_eq!(case.unwrap().phone, Phone::AA);
        }

        assert_eq!(LexicalStress::NoStress, cases[0].unwrap().stress);
        assert_eq!(LexicalStress::NoStress, cases[1].unwrap().stress);
        assert_eq!(LexicalStress::Primary, cases[2].unwrap().stress);
        assert_eq!(LexicalStress::Secondary, cases[3].unwrap().stress);
    }

    #[test]
    fn phoneme_fails_toparse_invalid_input() {
        assert_eq!(None, Phoneme::from("FOOBAR"));
        assert_eq!(None, Phoneme::from("0AA"));
        assert_eq!(None, Phoneme::from(" AA "));
    }

    #[test]
    fn phoneme_set_parses_line_properly() {
        let example_a = PhonemeSet::from("HYUN  HH AY1 AH0 N");
        assert!(example_a.is_some());
        let example_a = example_a.unwrap();

        assert_eq!(example_a.word, "HYUN");
        assert_eq!(example_a.set[0].phone, Phone::HH);
        assert_eq!(example_a.set[0].stress, LexicalStress::NoStress);

        assert_eq!(example_a.set[1].phone, Phone::AY);
        assert_eq!(example_a.set[1].stress, LexicalStress::Primary);

        assert_eq!(example_a.set[2].phone, Phone::AH);
        assert_eq!(example_a.set[2].stress, LexicalStress::NoStress);

        assert_eq!(example_a.set[3].phone, Phone::N);
        assert_eq!(example_a.set[3].stress, LexicalStress::NoStress);
    }
}
