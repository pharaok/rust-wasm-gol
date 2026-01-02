use std::fs;
use std::io::Write;

use gol::parse::rle;

fn main() {
    let mut patterns = Vec::new();
    for entry in fs::read_dir("public/patterns").unwrap() {
        let path = entry.unwrap().path();
        let bytes = fs::read(&path).unwrap();
        let rle = String::from_utf8_lossy(&bytes);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if let Ok((meta, _)) = rle::parse_metadata(&rle, file_name, file_name) {
            patterns.push(meta);
        }
    }

    let json = serde_json::to_string(&patterns).unwrap();
    let mut file = fs::File::create("public/patterns.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
