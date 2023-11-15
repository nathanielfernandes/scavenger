use scavenger::parse_path_str;

fn main() {
    let path = "L-134.2-123-23.4 18.2-26-20.3 77.2-60.1z";

    let commands = parse_path_str(path).unwrap();

    for cmd in commands {
        println!("{:?}", cmd);
    }
}
