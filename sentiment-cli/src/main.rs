use std::io;

fn main() -> io::Result<()> {
    let analyzer = vader_sentiment::SentimentIntensityAnalyzer::new();
    let mut buffer = String::new();
    loop {
        io::stdin().read_line(&mut buffer)?;
        buffer.pop();
        if buffer == "exit" || buffer == "\u{4}"{
            break;
        }
        print_sentiment(&buffer, &analyzer);
        println!("{:?}", buffer);
        buffer.clear();
    }
    Ok(())
}

fn print_sentiment(sentence: &str, analyzer: &vader_sentiment::SentimentIntensityAnalyzer) {
    let scores = analyzer.polarity_scores(sentence);
    println!("{:?}", scores);
}