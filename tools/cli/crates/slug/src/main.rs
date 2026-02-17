use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.first().is_some_and(|a| a == "--help") {
        println!(
            "Usage: slug [TOPIC ...]\n\n\
             Generate a URL-safe kebab-case slug from a topic string.\n\n\
             Arguments are joined into a single string, so quoting is optional:\n  \
             slug refactor auth module    # refactor-auth-module\n  \
             slug \"refactor auth module\"  # same result\n\n\
             Articles (a, an, the) and common filler words are stripped.\n\
             Output is truncated to 50 characters on a word boundary."
        );
        return;
    }

    if args.is_empty() {
        return;
    }

    let input = args.join(" ");
    let result = claude_slug::slug(&input);
    if !result.is_empty() {
        println!("{result}");
    }
}
