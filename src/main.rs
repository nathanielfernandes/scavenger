use scavenger::parse_path_str;

fn main() {
    let path = "M 10 315
    L 110 215
    A 36 60 0 0 1 150.71 170.29
    L 172.55 152.45
    A 30 50 -45 0 1 215.1 109.9
    L 315 10";

    let commands = parse_path_str(path).unwrap();

    for cmd in commands {
        println!("{:#?}", cmd);
    }
}
