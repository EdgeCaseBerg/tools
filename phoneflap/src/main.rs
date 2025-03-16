use phoneflap::*;

use std::{thread, time};
use std::collections::VecDeque;
use time::Duration;


fn main() {
    let definitions = parse_dictionary("./data/cmudict-0.7b.txt");
    let definition = definitions.iter().next().unwrap();
    println!("{:?}", definition);
    println!("{:?}", definition.1.vowel_count());

    let mut definitions_random = definitions.values();

    let delay_between_frames = Duration::from_millis(25);

    let mut face = SimpleFlaps::new();
    let mut words = vec!["HELLO", "BLOG", "READERS", "I", "HOPE", "YOU'RE", "WELL"];
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
                time_until_next_word = Duration::from_millis(1000) + face.time_left_before_finished_speaking();  
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
    FlapOpen(Duration),
    FlapClosed(Duration)
}

impl SimpleFaceState {

    fn add_time(&self, additional_time: Duration) -> Self {
        match self {
            SimpleFaceState::Neutral => SimpleFaceState::Neutral,
            SimpleFaceState::FlapOpen(previous) => SimpleFaceState::FlapOpen(*previous + additional_time),
            SimpleFaceState::FlapClosed(previous) => SimpleFaceState::FlapClosed(*previous + additional_time),
        }
    }

    fn from(phoneme_set: &PhonemeSet) -> VecDeque<SimpleFaceState> {
        let states = VecDeque::new();
        // THIS -> DH IH1 S
        // 50 100 because consanant = 50, vowel collapsed duration until next vowel
        // THIS'LL -> DH IS1 S AH0 L
        // 50 100 100

        let mut flip_flop = true;
        let mut next_state = |tuple: (&Phoneme, Duration)| {
            let (_, duration) = tuple; // potentialy we use the phoneme in the future
            let face = if flip_flop {
                SimpleFaceState::FlapOpen(duration)
            } else {
                SimpleFaceState::FlapClosed(duration)
            };
            flip_flop = !flip_flop;
            face
        };

        phoneme_set.set.iter().map(|phoneme| {
            (phoneme, Duration::from_millis(50))
        }).fold(states, |mut acc, phoneme_tuple| {
            match acc.pop_back() {
                None => acc.push_back(next_state(phoneme_tuple)),
                Some(previous_state) => {
                    if phoneme_tuple.0.phone.contains_vowel() {
                        acc.push_back(previous_state);
                        acc.push_back(next_state(phoneme_tuple));
                    } else {
                        let new_state = previous_state.add_time(phoneme_tuple.1);
                        acc.push_back(new_state);
                    }
                }
            };
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
        match self.state {
            SimpleFaceState::Neutral => "0 u 0",
            SimpleFaceState::FlapClosed(_) => "0 - 0",
            SimpleFaceState::FlapOpen(_) => "0 o 0",
        }.to_string()
    }

    fn tick(&mut self, since_last_tick: Duration) {
        match self.state {
            SimpleFaceState::Neutral => self.next_state(),
            SimpleFaceState::FlapOpen(time_left) => {
                let new_time_left = time_left.checked_sub(since_last_tick);
                match new_time_left {
                    None => self.next_state(),
                    Some(time_left) => {
                        self.state = SimpleFaceState::FlapOpen(time_left);
                    }
                }
            },
            SimpleFaceState::FlapClosed(time_left) => {
                let new_time_left = time_left.checked_sub(since_last_tick);
                match new_time_left {
                    None => self.next_state(),
                    Some(time_left) => {
                        self.state = SimpleFaceState::FlapClosed(time_left);
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
            SimpleFaceState::FlapOpen(duration) => {
                duration
            },
            SimpleFaceState::FlapClosed(duration) => {
                duration
            }
        };
        for message in &self.messages {
            match message {
                SimpleFaceState::Neutral => {},
                SimpleFaceState::FlapOpen(duration) => {
                    time_left += *duration;
                },
                SimpleFaceState::FlapClosed(duration) => {
                    time_left += *duration;
                }
            }
        }
        time_left
    }

}
