pub mod rle {
    use regex::Regex;

    pub struct PatternMetadata {
        pub name: Option<String>,
        pub comment: String,
        pub owner: Option<String>,
        pub width: usize,
        pub height: usize,
        pub rule: String,
    }

    pub fn parse_metadata(rle: &str) -> Result<(PatternMetadata, usize), ()> {
        let section_re = Regex::new(r"(?m)^#([a-zA-Z])(.*)$").unwrap();
        let header_re = Regex::new(
            r"(?m)^\s*x\s*=\s*(\d+)\s*,?\s*y\s*=\s*(\d+)\s*(?:,?\s*rule\s*=\s*(\S+)\s*)?$",
        )
        .unwrap();

        let mut name = None;
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
                    name = Some(line.trim().to_string());
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
                comment,
                owner,
                width,
                height,
                rule: rule.to_string(),
            },
            start,
        ))
    }
    pub fn to_rect(rle: &str) -> Result<Vec<Vec<u8>>, ()> {
        let item_re = Regex::new(r"\s*(\d*)([a-zA-Z\$\!])").unwrap();

        let (PatternMetadata { width, height, .. }, mut start) = parse_metadata(rle)?;

        let mut rect = vec![vec![0; width]; height];
        let (mut i, mut j) = (0, 0);
        while let Some(c) = item_re.captures_at(rle, start) {
            let (_, [count_str, tag]) = c.extract();
            let count = count_str.parse().unwrap_or(1);
            start = c.get(0).unwrap().end();
            match tag {
                "!" => break,
                "$" => {
                    i += count;
                    j = 0;
                }
                _ => {
                    for jj in 0..count {
                        match tag {
                            "b" | "B" => {}
                            _ => rect[i][j + jj] = 1,
                        };
                    }
                    j += count;
                }
            }
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
