use js_sys::{wasm_bindgen::JsValue, Reflect};

pub fn get_index(captures: &JsValue) -> usize {
    Reflect::get(captures, &JsValue::from("index"))
        .ok()
        .unwrap()
        .as_f64()
        .unwrap() as usize
}
pub mod rle {
    use super::get_index;
    use js_sys::RegExp;
    use serde::{Deserialize, Serialize};

    thread_local! {
            static ITEM_RE: RegExp = RegExp::new(r"\s*(\d*)([a-zA-Z\$\!])", "");
            static SECTION_RE: RegExp = RegExp::new(r"^#([a-zA-Z])(.*)$", "m");
            static HEADER_RE: RegExp = RegExp::new(
                r"^\s*x\s*=\s*(\d+)\s*,?\s*y\s*=\s*(\d+)\s*(?:,?\s*rule\s*=\s*(\S+)\s*)?$","m"
        );
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    pub struct PatternMetadata {
        pub name: String,
        pub path: String,
        pub comment: String,
        pub owner: Option<String>,
        pub width: usize,
        pub height: usize,
        pub rule: String,
    }

    pub fn parse_metadata(
        rle: &str,
        name: &str,
        path: &str,
    ) -> Result<(PatternMetadata, usize), ()> {
        let mut name = name.to_owned();
        let mut comment = String::new();
        let mut owner = None;
        let mut rule = "23/3".to_owned();

        let mut start = 0;
        while let Some(captures) = SECTION_RE.with(|re| re.exec(&rle[start..])) {
            let letter = captures.get(1).as_string().unwrap();
            let line = captures.get(2).as_string().unwrap();

            match letter.as_str() {
                "C" | "c" => {
                    comment.push_str(line.trim());
                    comment.push('\n');
                }
                "N" => {
                    name = line.trim().to_owned();
                }
                "O" => {
                    owner = Some(line.trim().to_owned());
                }
                "r" => {
                    rule = line.trim().to_owned();
                }
                _ => {}
            }
            start += get_index(&captures) + captures.get(0).as_string().unwrap().len();
        }

        let captures = match HEADER_RE.with(|re| re.exec(&rle[start..])) {
            Some(c) => c,
            None => return Err(()),
        };
        let width: usize = captures
            .get(1)
            .as_string()
            .unwrap()
            .as_str()
            .parse()
            .unwrap();
        let height: usize = captures
            .get(2)
            .as_string()
            .unwrap()
            .as_str()
            .parse()
            .unwrap();
        if let Some(m) = captures.get(3).as_string() {
            rule = m;
        }
        start += get_index(&captures) + captures.get(0).as_string().unwrap().len();
        Ok((
            PatternMetadata {
                name,
                path: path.to_owned(),
                comment,
                owner,
                width,
                height,
                rule,
            },
            start,
        ))
    }

    pub struct RLEIterator {
        rle: String,
        start: usize,
        count: usize,
        x: usize,
        y: usize,
    }
    impl RLEIterator {
        pub fn new(rle: &str) -> Result<Self, ()> {
            let (_, start) = parse_metadata(rle, "Unnamed Pattern", "")?;

            Ok(Self {
                rle: rle.to_owned(),
                start,
                count: 0,
                x: 0,
                y: 0,
            })
        }
    }
    impl Iterator for RLEIterator {
        type Item = (usize, usize);

        fn next(&mut self) -> Option<Self::Item> {
            while self.start < self.rle.len() && let Some(c) = ITEM_RE.with(|re| re.exec(&self.rle[self.start..])) {
                let count_str = c.get(1).as_string().unwrap();
                let tag = c.get(2).as_string().unwrap();
                let count = count_str.parse().unwrap_or(1);
                let start = self.start + get_index(&c) + c.get(0).as_string().unwrap().len();
                match tag.as_str() {
                    "!" => break,
                    "$" => {
                        self.y += count;
                        self.x = 0;
                        self.start = start;
                        self.count = 0;
                    }
                    "b" | "B" => {
                        self.x += count;
                        self.start = start;
                        self.count = 0;
                    }
                    _ => {
                        if self.count < count {
                            self.count += 1;
                            self.x += 1;
                            return Some((self.x - 1, self.y));
                        } else {
                            self.start = start;
                            self.count = 0;
                        }
                    }
                }
            }

            None
        }
    }
    pub fn iter_alive(rle: &str) -> Result<RLEIterator, ()> {
        RLEIterator::new(rle)
    }

    pub fn to_rect(rle: &str) -> Result<Vec<Vec<u8>>, ()> {
        let (PatternMetadata { width, height, .. }, _) =
            parse_metadata(rle, "Unnamed Pattern", "")?;
        let mut rect = vec![vec![0; width]; height];
        for (x, y) in iter_alive(rle)? {
            rect[y][x] = 1;
        }
        Ok(rect)
    }

    fn item(count: usize, value: u8) -> String {
        let count_str = if count > 1 {
            count.to_string()
        } else {
            "".to_string()
        };

        format!("{}{}", count_str, if value == 0 { 'b' } else { 'o' })
    }
    pub fn from_rect(grid: &Vec<Vec<u8>>) -> String {
        let mut rle = format!("x = {}, y = {}\n", grid[0].len(), grid.len());
        let mut line_len = 0;
        let mut push_item = |item: &str| {
            line_len += item.len();
            if line_len > 70 {
                rle.push('\n');
                line_len = 0;
            }
            rle.push_str(item);
        };

        let mut count = 0;
        for row in grid {
            let mut prev = row[0];
            for cell in row {
                if *cell != prev {
                    push_item(&item(count, prev));
                    prev = *cell;
                    count = 0;
                }
                count += 1;
            }
            if prev != 0 {
                push_item(&item(count, prev));
            }
            push_item("$");
            count = 0;
        }
        rle.pop();
        rle.push('!');
        rle
    }
}
