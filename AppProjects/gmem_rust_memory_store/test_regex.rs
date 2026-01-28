use regex::Regex;

fn main() {
    let re = Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*"|'([^'\\]*(?:\\.[^'\\]*)*)'|(\S+))"#).unwrap();
    
    let test_cases = vec![
        "add test memory",
        "add",
        "search --limit 5 test",
        "add \"quoted text\"",
        "add 'single quoted'",
    ];
    
    for test in test_cases {
        println!("Input: {:?}", test);
        let matches: Vec<_> = re.find_iter(test).collect();
        println!("Matches: {:?}", matches);
        let tokens: Vec<String> = matches.iter()
            .map(|m| {
                let token = m.as_str();
                token.replace(r#"\""#, "\"").replace(r#"\'"#, "'")
            })
            .collect();
        println!("Tokens: {:?}", tokens);
        println!();
    }
}
