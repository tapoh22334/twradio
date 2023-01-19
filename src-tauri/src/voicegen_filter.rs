use regex::Regex;

pub fn replace_retweet(input: &str) -> String {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"RT [^:]*:").unwrap();
    }

    RE.replace_all(input, "RT").to_string()
}

pub fn replace_url(input: &str) -> String {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"https?://[\w!\?/\+\-_~=;\.,\*&@#\$%\(\)'\[\]]+").unwrap();
    }

    RE.replace_all(input, "ユーアールエル略").to_string()
}
