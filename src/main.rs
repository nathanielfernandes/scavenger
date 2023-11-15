use scavenger::parse_path_str;

fn main() {
    let path = "H 2 20";

    let commands = parse_path_str(path).unwrap();

    for cmd in commands {
        println!("{:?}", cmd);
    }
}
