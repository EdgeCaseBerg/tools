use phoneflap::*;

use std::{thread, time};
use std::collections::VecDeque;
use time::Duration;


fn main() {
    let definitions = parse_dictionary("./data/cmudict-0.7b.txt");
    let definition = definitions.iter().next().unwrap();
    println!("{:?}", definition);
    println!("{:?}", definition.1.vowel_count());

    let definitions_random = definitions.values();

    let delay_between_frames = Duration::from_millis(10);

    let mut face = SimpleFlaps::new();
    let words = vec!["HELLO", "BLOG", "READERS", "I", "HOPE", "YOU'RE", "ENJOYING", "THE", "BLOGOSPHERE"];
    let iter = words.into_iter().map(|word| {
        let p = definitions.get(word).unwrap();
        p
    });
    let mut definitions_random = iter.chain(definitions_random);

    let mut time_until_next_word =  Duration::from_millis(1000) + face.time_left_before_finished_speaking();
    let mut now_saying = String::new();

    let mut buffer = String::new();
    loop {
        print!("\x1B[2J\x1B[H");
        print!("{}", buffer);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        thread::sleep(delay_between_frames);
        buffer.clear();
        
        // Note: should _actually_ compute this delta.
        face.tick(delay_between_frames);
        
        buffer.push_str(&face.display());
        buffer.push('\n');
        buffer.push_str("Now saying: ");
        buffer.push_str(&now_saying);
        buffer.push('\n');

        match time_until_next_word.checked_sub(delay_between_frames) {
            None => {
                time_until_next_word = Duration::from_millis(500) + face.time_left_before_finished_speaking();  
                if let Some(phoneme_set) = definitions_random.next() {
                    now_saying = phoneme_set.word.clone();
                    face.speak(phoneme_set);
                }
            },
            Some(time_left) => {
                time_until_next_word = time_left;
            }
        }
    }
}


enum SimpleFaceState {
    Neutral,
    Flap(String, Duration), // Shape, Duration of shape
}

impl SimpleFaceState {
    fn from(phoneme_set: &PhonemeSet) -> VecDeque<SimpleFaceState> {
        let states = VecDeque::new();
        phoneme_set.set.iter().map(|phoneme| {
            (phoneme, Duration::from_millis(50))
        }).fold(states, |mut acc, phoneme_tuple| {
            acc.push_back({
                let mouth = phoneme_tuple.0.phone.to_mouth_shape();
                SimpleFaceState::Flap(mouth, phoneme_tuple.1)
            });
            acc
        })
    }
}

struct SimpleFlaps {
    state: SimpleFaceState,
    messages: VecDeque<SimpleFaceState>
}

impl SimpleFlaps {
    fn new() -> Self {
        Self {
            state: SimpleFaceState::Neutral,
            messages: VecDeque::new(),
        }
    }

    fn display(&self) -> String {
        match &self.state {
            SimpleFaceState::Neutral => "0 u 0".to_string(),
            SimpleFaceState::Flap(mouth, _) => format!("0 {} 0", &mouth),
        }
    }

    fn tick(&mut self, since_last_tick: Duration) {
        match &self.state {
            SimpleFaceState::Neutral => self.next_state(),
            SimpleFaceState::Flap(mouth, time_left) => {
                let new_time_left = time_left.checked_sub(since_last_tick);
                match new_time_left {
                    None => self.next_state(),
                    Some(time_left) => {
                        self.state = SimpleFaceState::Flap(mouth.to_string(), time_left);
                    }
                }
            }
        }
    }

    fn next_state(&mut self) {
        if let Some(new_state) = self.messages.pop_front() {
            self.state = new_state;
        } else {
            self.state = SimpleFaceState::Neutral;
        }
    }

    fn speak(&mut self, phoneme_set: &PhonemeSet) {
        let states = SimpleFaceState::from(phoneme_set);
        for state in states {
            self.messages.push_back(state);
        }
    }

    fn time_left_before_finished_speaking(&self) -> Duration {
        let mut time_left = match self.state {
            SimpleFaceState::Neutral => Duration::from_millis(0),
            SimpleFaceState::Flap(_, duration) => {
                duration
            }
        };
        for message in &self.messages {
            match message {
                SimpleFaceState::Neutral => {},
                SimpleFaceState::Flap(_, duration) => {
                    time_left += *duration;
                },
            }
        }
        time_left
    }

}
