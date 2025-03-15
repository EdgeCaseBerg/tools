use bstr::ByteSlice;
use std::fs;
use std::{borrow::Cow, collections::HashMap};

fn main() {
    let definitions = parse_dictionary("./data/cmudict-0.7b.txt");
    println!("{:?}", definitions.iter().next());
}

fn parse_dictionary(dictionary_file_path: &str) -> HashMap<String, PhonemeSet> {
    let not_utf8: Vec<u8> =
        fs::read(dictionary_file_path).expect("Could not load dictionary file");
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
struct PhonemeSet {
    set: Vec<Phoneme>,
    word: String,
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
        raw_line_data
            .split(" ")
            .filter_map(Phoneme::from)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Phoneme {
    phone: Phone,
    stress: LexicalStress,
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
            }
        };

        let raw_phone: String = phone_and_stress
            .chars()
            .take_while(|c| !c.is_ascii_digit())
            .collect();

        Phone::from(&raw_phone).map(|phone| {
            Self {
                phone,
                stress,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
enum Phone {
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
    fn from(str: &str) -> Option<Phone> {
        match str {
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
                println!("no match on {:?}", str);
                None
            }
        }
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, Copy)]
enum LexicalStress {
    NoStress, // kind of want to use None but don't want to get confused!
    Primary,
    Secondary,
}
