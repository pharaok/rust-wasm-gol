use js_sys::{Reflect, wasm_bindgen::JsValue};

pub fn get_index(captures: &JsValue) -> usize {
    Reflect::get(captures, &JsValue::from("index"))
        .ok()
        .unwrap()
        .as_f64()
        .unwrap() as usize
}
pub mod rle {
    use crate::universe::UniverseIterator;

    use super::get_index;
    use js_sys::RegExp;
    use serde::{Deserialize, Serialize};

    thread_local! {
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
        pub width: u32,
        pub height: u32,
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
        let width = captures
            .get(1)
            .as_string()
            .unwrap()
            .as_str()
            .parse()
            .unwrap();
        let height = captures
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

    pub struct RLEIterator<'a> {
        rle: &'a str,
        i: usize,
        count: usize,
        x: i64,
        y: i64,
    }
    impl<'a> RLEIterator<'a> {
        pub fn new(rle: &'a str) -> Result<Self, ()> {
            let (_, start) = parse_metadata(rle, "Unnamed Pattern", "")?;

            Ok(Self {
                rle: &rle[start..],
                i: 0,
                count: 0,
                x: 0,
                y: 0,
            })
        }
    }
    impl<'a> Iterator for RLEIterator<'a> {
        type Item = (i64, i64);

        fn next(&mut self) -> Option<Self::Item> {
            if self.count > 0 {
                self.count -= 1;
                self.x += 1;
                return Some((self.x - 1, self.y));
            }

            let bytes = self.rle.as_bytes();
            loop {
                while self.i < bytes.len() && bytes[self.i].is_ascii_whitespace() {
                    self.i += 1;
                }
                if self.i >= bytes.len() {
                    return None;
                }

                let mut count_str_end = self.i;
                while count_str_end < bytes.len() && bytes[count_str_end].is_ascii_digit() {
                    count_str_end += 1;
                }
                self.count = str::from_utf8(&bytes[self.i..count_str_end])
                    .unwrap()
                    .parse()
                    .unwrap_or(1);
                self.i = count_str_end;
                if self.i >= bytes.len() {
                    return None;
                }

                let tag = bytes[self.i] as char;
                self.i += 1;
                match tag {
                    '!' => return None,
                    '$' => {
                        self.y += self.count as i64;
                        self.x = 0;
                        self.count = 0;
                    }
                    'b' | 'B' => {
                        self.x += self.count as i64;
                        self.count = 0;
                    }
                    _ => {
                        self.count -= 1;
                        self.x += 1;
                        return Some((self.x - 1, self.y));
                    }
                }
            }
        }
    }
    pub fn iter_alive<'a>(rle: &'a str) -> Result<RLEIterator<'a>, ()> {
        RLEIterator::new(rle)
    }

    pub fn to_grid(rle: &str) -> Result<Vec<Vec<u8>>, ()> {
        let (PatternMetadata { width, height, .. }, _) =
            parse_metadata(rle, "Unnamed Pattern", "")?;
        let mut rect = vec![vec![0; width as usize]; height as usize];
        for (x, y) in iter_alive(rle)? {
            rect[y as usize][x as usize] = 1;
        }
        Ok(rect)
    }

    fn item(count: i64, value: &str) -> String {
        let count_str = if count > 1 {
            count.to_string()
        } else {
            "".to_string()
        };

        count_str + value
    }
    pub fn from_iter(iter: UniverseIterator, x1: i64, y1: i64, x2: i64, y2: i64) -> String {
        let mut cells = iter.collect::<Vec<_>>();
        cells.sort_by_key(|&(x, y)| (y, x));
        let (width, height) = (x2 - x1 + 1, y2 - y1 + 1);

        let mut items = Vec::new();
        let (mut px, mut py) = (-1, 0);
        let mut run = 0;
        for (mut x, mut y) in cells {
            x -= x1;
            y -= y1;
            let dy = y - py;
            if dy > 0 {
                if run > 0 {
                    items.push(item(run, "o"));
                    run = 0;
                }
                items.push(item(dy, "$"));
                px = -1;
            }
            let dx = x - px - 1;
            if dx > 0 {
                if run > 0 {
                    items.push(item(run, "o"));
                    run = 0;
                }
                items.push(item(dx, "b"));
            }
            run += 1;

            px = x;
            py = y;
        }
        if run > 0 {
            items.push(item(run, "o"));
        }
        while !items.is_empty() && items.last().unwrap().ends_with('b') {
            items.pop();
        }
        items.push("!".to_owned());

        let mut rle = format!("x = {}, y = {}\n", width, height);
        let mut line_len = 0;
        for item in items.iter() {
            line_len += item.len();
            if line_len > 70 {
                rle.push('\n');
                line_len = item.len();
            }
            rle.push_str(item);
        }
        rle
    }
}
