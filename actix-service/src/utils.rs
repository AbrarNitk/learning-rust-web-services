/// Parse argument from CLI
/// If CLI command: fpm serve --identities a@foo.com,foo
/// key: --identities -> output: a@foo.com,foo
pub fn parse_from_cli(key: &str) -> Option<String> {
    use itertools::Itertools;
    let args = std::env::args().collect_vec();
    let mut index = None;
    for (idx, arg) in args.iter().enumerate() {
        if arg.eq(key) {
            index = Some(idx);
        }
    }
    index
        .and_then(|idx| args.get(idx + 1))
        .map(String::to_string)
}
