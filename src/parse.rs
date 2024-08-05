pub mod rle {
    use leptos::logging::log;
    use regex::Regex;

    pub fn to_rect(rle: &str) -> Result<Vec<Vec<u8>>, ()> {
        let section_re = Regex::new(r"(?m)^#([a-zA-Z])(.*)$").unwrap();
        let header_re = Regex::new(
            r"(?m)^\s*x\s*=\s*(\d+)\s*,?\s*y\s*=\s*(\d+)\s*(?:,\s*rule\s*=\s*(B\d*\/S\d*)\s*)?$",
        )
        .unwrap();
        let rle_re = Regex::new(r"\s*(?:([\$\!])|(\d+)?([a-zA-Z]))").unwrap();

        let mut rule = "23/3";

        let mut start = 0;
        while let Some(c) = section_re.captures_at(rle, start) {
            let (_, [letter, line]) = c.extract();
            log!("{:?} {:?}", letter, line);
            start = c.get(0).unwrap().end();
        }

        let captures = match header_re.captures(rle) {
            Some(c) => c,
            None => return Err(()),
        };
        let w: usize = captures.get(1).unwrap().as_str().parse().unwrap();
        let h: usize = captures.get(2).unwrap().as_str().parse().unwrap();
        if let Some(m) = captures.get(3) {
            rule = m.as_str()
        }
        start = captures.get(0).unwrap().end();

        let mut grid = vec![vec![0; w]; h];
        let mut i = 0;
        let mut j = 0;
        while let Some(c) = rle_re.captures_at(rle, start) {
            if let Some(end) = c.get(1).map(|m| m.as_str()) {
                if end == "!" {
                    break;
                }
                if end == "$" {
                    i += 1;
                    j = 0;
                }
                start = c.get(0).unwrap().end();
                continue;
            }
            let count: usize = c.get(2).map(|m| m.as_str()).unwrap_or("1").parse().unwrap();
            let tag = c.get(3).unwrap().as_str();
            for jj in 0..count {
                grid[i][j + jj] = match tag {
                    "b" => 0,
                    "B" => 0, // (and perhaps B)
                    _ => 1,
                };
            }
            j += count;

            start = c.get(0).unwrap().end();
        }

        Ok(grid)
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
