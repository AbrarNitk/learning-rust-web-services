lazy_static::lazy_static! {
    static ref PATH_PARAMS: regex::Regex = regex::Regex::new(r"<\s*([a-z]\w+)\s*:\s*([a-z|A-Z|0-9|_]\w+)\s*>").unwrap();
}


fn main() {
//    let regex: regex::Regex = regex::Regex::new(r"<\s*([a-z]\w+)\s*:\s*([a-z|A-Z|0-9|_]\w+)\s*>").unwrap();
    let text = "/books/<string:name>/<integer:a_ge>";
    for cap in PATH_PARAMS.captures_iter(text) {
        println!("{:?}", cap);
    }
    println!("Hello World");
}