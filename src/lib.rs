use std::collections::HashMap;

const M: usize = 1e9 as usize + 7;

fn delta(a: &[u8], b: &[u8], min_match_len: usize) -> Vec<((usize, usize), (usize, usize))> {
    let hash_len = (min_match_len + 1) / 2;
    let hashes: HashMap<usize, usize> = RollingHash::new(a, hash_len, hash_len).collect();
    println!("{hashes:?}");
    let matches = RollingHash::new(b, hash_len, 1).filter_map(|(hb, ib)| {
        if let Some(&ia) = hashes.get(&hb) {
            Some(expand_match(a, b, ia, ib))
        } else {
            None
        }
    });
    let x: Vec<_> = matches.collect();
    println!("{x:?}");
    let x = merge_maches(&x);
    println!("{x:?}");
    x
}

struct RollingHash<'a> {
    data: &'a [u8],
    hash_len: usize,
    stride: usize,
    index: usize,
    hash: Option<usize>,
}

impl<'a> RollingHash<'a> {
    pub fn new(data: &'a [u8], hash_len: usize, stride: usize) -> Self {
        let hash_len = std::cmp::min(data.len(), hash_len);
        Self {
            data,
            stride,
            hash_len,
            index: 0,
            hash: None,
        }
    }

    fn calc_initial_hash(data: &[u8], hash_len: usize) -> usize {
        let mut hash = 0;
        for i in 0..hash_len {
            hash = hash + modpow(Self::B, hash_len - 1 - i) * Self::to_usize(data[i]);
            hash %= M;
        }
        hash
    }

    fn to_usize(x: u8) -> usize {
        x as usize + 1
    }

    const B: usize = 100;
}

impl<'a> Iterator for RollingHash<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.hash_len + self.stride > self.data.len() {
            return None;
        }

        if let None = self.hash {
            let hash = Self::calc_initial_hash(self.data, self.hash_len);
            self.hash = Some(hash);
            return Some((hash, 0));
        }

        let mut hash = self.hash.unwrap();
        for i in 0..self.stride {
            let i = self.index + i;
            let v1 = Self::B * hash % M;
            let v2 = Self::to_usize(self.data[i + self.hash_len]);
            let v3 = modpow(Self::B, self.hash_len) * Self::to_usize(self.data[i]) % M;
            // v1 + v2 - v3
            hash = (v1 + v2 + M - v3) % M;
        }

        self.index += self.stride;
        self.hash = Some(hash);
        Some((hash, self.index))
    }
}

fn modpow(a: usize, b: usize) -> usize {
    if b == 0 {
        return 1;
    }

    let a = a % M;
    if b % 2 == 0 {
        modpow(a * a, b / 2) % M
    } else {
        a * modpow(a, b - 1) % M
    }
}

fn expand_match(a: &[u8], b: &[u8], ia: usize, ib: usize) -> ((usize, usize), (usize, usize)) {
    let r = a[ia..]
        .iter()
        .zip(&b[ib..])
        .take_while(|(va, vb)| va == vb)
        .count();

    let l = a[..ia]
        .iter()
        .rev()
        .zip(b[..ib].iter().rev())
        .take_while(|(va, vb)| va == vb)
        .count();

    ((ia - l, ia + r - 1), (ib - l, ib + r - 1))
}

fn merge_maches(
    matches: &[((usize, usize), (usize, usize))],
) -> Vec<((usize, usize), (usize, usize))> {
    let mut matches = Vec::from(matches);
    matches.sort_by(|x, y| x.1 .0.cmp(&y.1 .0));
    let matches = matches.iter().scan(0, |rp, &((la, ra), (lb, rb))| {
        if *rp <= lb {
            *rp = rb + 1;
            return Some(((la, ra), (lb, rb)));
        }
        if rb < *rp {
            return None;
        }
        let la = la + (*rp - lb);
        let lb = *rp;
        *rp = rb + 1;
        Some(((la, ra), (lb, rb)))
    });
    matches.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_abcd() {
        let result = delta("xxabcdxx".as_bytes(), "abcd".as_bytes(), 4);
        assert_eq!(result, vec![((2, 5), (0, 3))]);
    }

    #[test]
    fn merge_matches_1() {
        let matches = [((0, 5), (0, 5)), ((3, 9), (3, 9))];
        let results = merge_maches(&matches);
        assert_eq!(vec![((0, 5), (0, 5)), ((6, 9), (6, 9))], results);
    }

    #[test]
    fn merge_matches_2() {
        let matches = [((0, 10), (5, 15)), ((0, 5), (5, 10)), ((20, 20), (15, 15))];
        let results = merge_maches(&matches);
        assert_eq!(vec![((0, 10), (5, 15))], results);
    }

    #[test]
    fn expand_maches_1() {
        let result = expand_match("xxabcdxx".as_bytes(), "abcd".as_bytes(), 2, 0);
        assert_eq!(result, ((2, 5), (0, 3)));
    }

    #[test]
    fn modpow_31_41() {
        let result = modpow(31, 41);
        assert_eq!(result, 411956758);
    }

    #[test]
    fn rolling_hash_0101() {
        let mut hashes = RollingHash::new(&[0, 1, 0, 1], 3, 1);
        assert_eq!(hashes.next(), Some((10201, 0)));
        assert_eq!(hashes.next(), Some((20102, 1)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_010101() {
        let mut hashes = RollingHash::new(&[0, 1, 0, 1, 0, 1], 3, 1);
        assert_eq!(hashes.next(), Some((10201, 0)));
        assert_eq!(hashes.next(), Some((20102, 1)));
        assert_eq!(hashes.next(), Some((10201, 2)));
        assert_eq!(hashes.next(), Some((20102, 3)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_abcdefg() {
        let mut hashes = RollingHash::new("abcdefg".as_ref(), 4, 2);
        let expected = [99000101, 100010202, 101020303, 102030404];
        assert_eq!(hashes.next(), Some((expected[0], 0)));
        assert_eq!(hashes.next(), Some((expected[2], 2)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_exceeds_mod() {
        let data = vec![255u8; 20];
        let mut hashes = RollingHash::new(&data, 10, 3);
        assert_eq!(hashes.next(), Some((757588431, 0)));
        assert_eq!(hashes.next(), Some((757588431, 3)));
        assert_eq!(hashes.next(), Some((757588431, 6)));
        assert_eq!(hashes.next(), Some((757588431, 9)));
        assert_eq!(hashes.next(), None);
    }
}
