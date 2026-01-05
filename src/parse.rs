pub mod rle {
    use regex::Regex;
    use serde::{Deserialize, Serialize};

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
        let section_re = Regex::new(r"(?m)^#([a-zA-Z])(.*)$").unwrap();
        let header_re = Regex::new(
            r"(?m)^\s*x\s*=\s*(\d+)\s*,?\s*y\s*=\s*(\d+)\s*(?:,?\s*rule\s*=\s*(\S+)\s*)?$",
        )
        .unwrap();

        let mut name = name.to_owned();
        let mut comment = String::new();
        let mut owner = None;
        let mut rule = "23/3";

        let mut start = 0;
        while let Some(c) = section_re.captures_at(rle, start) {
            let (_, [letter, line]) = c.extract();
            match letter {
                "C" | "c" => {
                    comment.push_str(line.trim());
                    comment.push('\n');
                }
                "N" => {
                    name = line.trim().to_string();
                }
                "O" => {
                    owner = Some(line.trim().to_string());
                }
                "r" => {
                    rule = line.trim();
                }
                _ => {}
            }
            start = c.get(0).unwrap().end();
        }

        let captures = match header_re.captures(rle) {
            Some(c) => c,
            None => return Err(()),
        };
        let width: usize = captures.get(1).unwrap().as_str().parse().unwrap();
        let height: usize = captures.get(2).unwrap().as_str().parse().unwrap();
        if let Some(m) = captures.get(3) {
            rule = m.as_str();
        }
        start = captures.get(0).unwrap().end();
        Ok((
            PatternMetadata {
                name,
                path: path.to_owned(),
                comment,
                owner,
                width,
                height,
                rule: rule.to_string(),
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
            let (PatternMetadata { width, height, .. }, start) =
                parse_metadata(rle, "Unnamed Pattern", "")?;

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
            let item_re = Regex::new(r"\s*(\d*)([a-zA-Z\$\!])").unwrap();

            while let Some(c) = item_re.captures_at(&self.rle, self.start) {
                let (_, [count_str, tag]) = c.extract();
                let count = count_str.parse().unwrap_or(1);
                let start = c.get(0).unwrap().end();
                match tag {
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
    pub fn iter(rle: &str) -> Result<RLEIterator, ()> {
        RLEIterator::new(rle)
    }

    pub fn to_rect(rle: &str) -> Result<Vec<Vec<u8>>, ()> {
        let (PatternMetadata { width, height, .. }, _) =
            parse_metadata(rle, "Unnamed Pattern", "")?;
        let mut rect = vec![vec![0; width]; height];
        for (x, y) in iter(rle)? {
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
