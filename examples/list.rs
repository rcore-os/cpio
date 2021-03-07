use std::io::Read;

use cpio::CpioNewcReader;

fn main() {
    let path = std::env::args().nth(1).expect("usage: list <cpio_path>");
    let mut file = std::fs::File::open(path).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();
    for e in CpioNewcReader::new(&content) {
        let (_, name, _) = e.unwrap();
        println!("{}", name);
    }
}
