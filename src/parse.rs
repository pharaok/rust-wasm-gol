use leptos::logging::log;
use regex::Regex;

use crate::quadtree::Node;

impl Node {
    pub fn rle_to_rect(rle: &str) -> Vec<Vec<u8>> {
        let section_re = Regex::new(r"(?m)^#([a-zA-Z])(.*)$").unwrap();
        let header_re = Regex::new(
            r"(?m)^x\s*=\s*(\d+)\s*,?\s*y\s*=\s*(\d+)\s*(?:,\s*rule\s*=\s*(B\d*\/S\d*)\s*)?$",
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

        let captures = header_re.captures(rle).unwrap();
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

        grid
    }
}
