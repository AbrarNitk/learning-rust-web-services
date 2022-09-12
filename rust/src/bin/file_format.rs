
// If a file contains line more than 80 chars, this program will append that to next line
// cargo run --bin file_format -- --file src/bin/temp.ftd
// Or
// cargo install path=.
// file_format --file `pwd`/src/bin/temp.ftd

/// Parse argument from CLI
/// If CLI command: fpm serve --identities a@foo.com,foo
/// key: --identities -> output: a@foo.com,foo
pub fn parse_from_cli(key: &str) -> Option<String> {
    let args = std::env::args().collect::<Vec<_>>();
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


fn find_space(l: &str, count: usize) -> Option<usize>{
    if l.len() <= count {
        return None;
    }
    let cs = l.chars().collect::<Vec<_>>();
    // Find first space before count
    let mut index = count;
    while index > 0 {
        if cs[index] == ' ' {
            return Some(index)
        }
        index -= 1;
    }

    // Find first space after count
    let mut index = count;
    let len = cs.len();
    while index < len {
        if cs[index] == ' ' {
            return Some(index)
        }
        index += 1;
    }
    None
}

fn line_split(l: &str, count: usize) -> (&str, Option<&str>){
    if let Some(s) = find_space(l, count) {
        return (&l[0..s], Some(&l[s..]))
    }
    (l, None)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = parse_from_cli("--file").unwrap();
    println!("Formatting: {}", file_path);
    // let current_dir = std::env::current_dir()?;
    let file_path = std::path::Path::new(&file_path); //current_dir.join(relative_file_path);
    let file_content = String::from_utf8(tokio::fs::read(&file_path).await?)?;
    let mut new_content = String::new();
    let mut remaining = "".to_string();
    for line in file_content.split("\n") {
        let line = if remaining.is_empty() {
            line.to_string()
        } else {
            format!("{} {}", remaining, line)
        };
        let (formatted_line, new_remaining) = line_split(&line, 80);
        new_content.push_str(&format!("{}\n", formatted_line));
        remaining = new_remaining.map(|x| x.trim_start().to_string()).unwrap_or("".to_string());
    }
    if !remaining.is_empty() {
        new_content.push_str(&format!("{}\n", remaining));
    }

    tokio::fs::write(file_path, new_content).await?;

    // println!("{}", new_content);
    Ok(())
}

/*
If a line is greater than 80 chars, break line by reading words, and then break it at space
append it to next line with trim space if any.
 */

// Note:
// How to check if file path is relative or from root.
// File or directory watcher


/*
# Bugs

## With the below content

- https://www.osho.com/osho-online-library/osho-talks/truth-significance-longing-a6a1ce12-16e?p=8996eacbcc7d1c777f16bfb1b21ee608
- Path of Meditation: https://www.amazon.in/Path-Meditation-Step-step-Guide/dp/8172610718
- https://www.amazon.in/Beyond-Psychology-Talks-Uruguay-Osho/dp/8172611951

 */