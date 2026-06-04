fn classify(input: i32) -> &'static str {
    if input % 2 == 0 {
        "even"
    } else {
        "odd"
    }
}

fn main() {
    let input = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    println!("{}", classify(input));
}
