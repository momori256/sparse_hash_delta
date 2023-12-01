use std::collections::HashMap;

const M: usize = 1e9 as usize + 7;

fn delta(a: &[u8], b: &[u8], min_match_len: usize) -> Vec<MatchInterval> {
    let hash_len = (min_match_len + 1) / 2;
    let hashes: HashMap<usize, usize> = RollingHash::new(a, hash_len, hash_len).collect();
    let matches = RollingHash::new(b, hash_len, 1)
        .filter_map(|(hb, ib)| {
            if let Some(&ia) = hashes.get(&hb) {
                Some(MatchInterval::new(a, b, ia, ib))
            } else {
                None
            }
        })
        .scan(MatchInterval::empty(), |acc, mut m| {
            m.remove_overlap(acc);
            if m.len > 0 {
                *acc = m;
                Some(m)
            } else {
                None
            }
        });
    matches.collect()
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
            hash = (v1 + v2 + M - v3) % M; // v1 + v2 - v3
        }

        self.index += self.stride;
        self.hash = Some(hash);
        Some((hash, self.index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MatchInterval {
    la: usize,
    lb: usize,
    len: usize,
}

impl MatchInterval {
    fn new(a: &[u8], b: &[u8], ia: usize, ib: usize) -> Self {
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

        let al = ia - l;
        let bl = ib - l;
        let len = l + r;
        Self {
            la: al,
            lb: bl,
            len,
        }
    }

    fn empty() -> Self {
        Self {
            la: 0,
            lb: 0,
            len: 0,
        }
    }

    fn br(&self) -> usize {
        self.lb + self.len
    }

    fn remove_overlap(&mut self, other: &Self) {
        if other.br() <= self.lb {
            return;
        }
        if other.lb <= self.lb && self.br() <= other.br() {
            self.len = 0;
            return;
        }

        let diff = other.br() - self.lb + 1;
        self.len -= diff;
        self.la += diff;
        self.lb += diff;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn init_match_interval(la: usize, lb: usize, len: usize) -> MatchInterval {
        MatchInterval { la, lb, len }
    }

    #[test]
    fn delta_abcd() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [2, 3, 4, 5];
        let result = delta(&a, &b, 4);
        assert_eq!(result, vec![init_match_interval(2, 0, 4)]);
    }

    #[test]
    fn match_interval_new() {
        let a = [0, 1, 2, 3, 4, 5];
        let b = [2, 3, 4];
        let result = MatchInterval::new(&a, &b, 3, 1);
        assert_eq!(result, init_match_interval(2, 0, 3));
    }

    #[test]
    fn match_interval_remove_overlap_partial() {
        // m1 : |--------|
        // m2 :      |--------|
        // m2':           |---|
        let m1 = init_match_interval(0, 0, 10);
        let mut m2 = init_match_interval(3, 5, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, init_match_interval(9, 11, 4));
    }

    #[test]
    fn match_interval_remove_overlap_all() {
        // m1 : |--------|
        // m2 :   |------|
        // m2':   ||
        let m1 = init_match_interval(0, 0, 10);
        let mut m2 = init_match_interval(3, 5, 5);
        m2.remove_overlap(&m1);
        assert_eq!(m2, init_match_interval(3, 5, 0));
    }

    #[test]
    fn match_interval_remove_overlap_same() {
        // m1 : |--------|
        // m2 : |--------|
        // m2': ||
        let m1 = init_match_interval(0, 0, 10);
        let mut m2 = init_match_interval(0, 0, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, init_match_interval(0, 0, 0));
    }

    #[test]
    fn match_interval_remove_overlap_empty() {
        // m1 : ||
        // m2 : |--------|
        // m2': |--------|
        let m1 = MatchInterval::empty();
        let mut m2 = init_match_interval(0, 0, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, init_match_interval(0, 0, 10));
    }

    #[test]
    fn match_interval_remove_overlap_none() {
        // m1 : |--------|
        // m2 :           |--------|
        // m2':           |--------|
        let m1 = init_match_interval(0, 0, 10);
        let mut m2 = init_match_interval(3, 11, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, init_match_interval(3, 11, 10));
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
