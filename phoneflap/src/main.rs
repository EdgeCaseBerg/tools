use phoneflap::*;

fn main() {
    let definitions = parse_dictionary("./data/cmudict-0.7b.txt");
    let definition = definitions.iter().next().unwrap();
    println!("{:?}", definition);
    println!("{:?}", definition.1.vowel_count());
}
