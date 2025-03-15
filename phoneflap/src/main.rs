use std::fs;
use bstr::ByteSlice;
use std::{borrow::Cow, collections::HashMap};

fn main() {
    let definitions = parse_dictionary("./data/cmudict-0.7b.txt");
    println!("{:?}", definitions.iter().next());
}

fn parse_dictionary(dictionary_file_path: &str) -> HashMap<String, PhonemeSet> {
    let not_utf8: Vec<u8> = std::fs::read(dictionary_file_path).expect("Could not load dictionary file");
    let definitions: HashMap<String, PhonemeSet> = not_utf8
        .lines()
        .map(decode)
        .filter_map(|line| PhonemeSet::new(&line).map(|p| (line.into(), p)))
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
}

impl PhonemeSet {
    fn new(raw_line: &str) -> Option<Self> {
        Some(PhonemeSet { set: Vec::new() })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Phoneme {
    phone: Phone,
    stress: LexicalStress
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
enum Phone {
  AA,
  AE,
  AH,
  AO,
  AW,
  AY,
  B ,
  CH,
  D ,
  DH,
  EH,
  ER,
  EY,
  F ,
  G ,
  HH,
  IH,
  IY,
  JH,
  K ,
  L ,
  M ,
  N ,
  NG,
  OW,
  OY,
  P ,
  R ,
  S ,
  SH,
  T ,
  TH,
  UH,
  UW,
  V ,
  W ,
  Y ,
  Z ,
  ZH,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, Copy)]
enum LexicalStress {
    NoStress, // kind of want to use None but don't want to get confused!
    Primary,
    Secondary
}