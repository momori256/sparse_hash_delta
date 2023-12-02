use std::collections::HashMap;

const M: usize = 1e9 as usize + 7;
const B: usize = 100;

#[derive(Debug, PartialEq)]
pub enum Compression<'a> {
    Match(usize, usize),
    Raw(&'a [u8]),
}

pub fn delta<'a>(a: &'a [u8], b: &'a [u8], min_match_len: usize) -> Vec<Compression<'a>> {
    use Compression::*;

    let match_intervals = extract_matches(a, b, min_match_len);
    if match_intervals.is_empty() {
        return vec![Raw(b)];
    }

    let mut results = Vec::with_capacity(match_intervals.len());
    let mut prev = 0;
    for MatchInterval { la, lb, len } in match_intervals {
        if prev < lb {
            results.push(Raw(&b[prev..lb]));
        }
        results.push(Match(la, len));
        prev = lb + len;
    }
    if prev != b.len() {
        results.push(Raw(&b[prev..]));
    }
    results
}

pub fn restore<'a>(a: &'a [u8], compressions: &[Compression<'a>]) -> Vec<&'a [u8]> {
    let mut results = Vec::new();
    for c in compressions {
        match c {
            Compression::Match(la, len) => {
                results.push(&a[*la..*la + *len]);
            }
            Compression::Raw(data) => {
                results.push(*data);
            }
        }
    }
    results.into_iter().collect()
}

fn extract_matches(a: &[u8], b: &[u8], min_match_len: usize) -> Vec<MatchInterval> {
    let hash_len = (min_match_len + 1) / 2;
    let hashes: HashMap<usize, usize> = RollingHash::new(a, hash_len).step_by(hash_len).collect();

    let matches = RollingHash::new(b, hash_len)
        .scan(0, |state, (hb, ib)| {
            if ib < *state {
                return Some(MatchInterval::empty());
            }
            if let Some(&ia) = hashes.get(&hb) {
                let m = MatchInterval::new(a, b, ia, ib);
                *state = m.br();
                Some(m)
            } else {
                Some(MatchInterval::empty())
            }
        })
        .scan(MatchInterval::empty(), |acc, mut m| {
            m.remove_overlap(acc);
            if m.len > 0 {
                *acc = m;
            }
            Some(m)
        })
        .filter(|m| m.len > 0);

    matches.collect()
}

pub struct RollingHash<'a> {
    data: &'a [u8],
    hash_len: usize,
    index: usize,
    hash: Option<usize>,
    base_pow: usize,
}

impl<'a> RollingHash<'a> {
    pub fn new(data: &'a [u8], hash_len: usize) -> Self {
        let hash_len = std::cmp::min(data.len(), hash_len);
        let base_pow = modpow(B, hash_len);
        Self {
            data,
            hash_len,
            index: 0,
            hash: None,
            base_pow,
        }
    }

    fn initial_hash(data: &[u8], hash_len: usize) -> usize {
        data.iter()
            .take(hash_len)
            .fold(0, |hash, &byte| (hash * B + Self::to_usize(byte)) % M)
    }

    fn to_usize(x: u8) -> usize {
        x as usize + 1
    }
}

impl<'a> Iterator for RollingHash<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.hash_len >= self.data.len() {
            return None;
        }

        if self.hash.is_none() {
            let hash = Self::initial_hash(self.data, self.hash_len);
            self.hash = Some(hash);
            return Some((hash, 0));
        }

        let v1 = B * self.hash.unwrap() % M;
        let v2 = Self::to_usize(self.data[self.index + self.hash_len]);
        let v3 = self.base_pow * Self::to_usize(self.data[self.index]) % M;
        let hash = (v1 + v2 + M - v3) % M; // v1 + v2 - v3

        self.index += 1;
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
    // Search the matching interval from a[ia] and b[ib].
    // a[la..la+len] == b[lb..lb+len].
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

        let la = ia - l;
        let lb = ib - l;
        let len = l + r;
        Self { la, lb, len }
    }

    fn empty() -> Self {
        static EMPTY: MatchInterval = MatchInterval {
            la: 0,
            lb: 0,
            len: 0,
        };
        EMPTY
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
        self.len = self.len.saturating_sub(diff);
        self.la += diff;
        self.lb += diff;
    }
}

fn modpow(base: usize, exponent: usize) -> usize {
    let mut result = 1;
    let mut base = base;
    let mut exponent = exponent;
    while exponent > 0 {
        if exponent % 2 == 1 {
            result = (result * base) % M;
        }
        base = (base * base) % M;
        exponent /= 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_match_interval(la: usize, lb: usize, len: usize) -> MatchInterval {
        MatchInterval { la, lb, len }
    }

    #[test]
    fn extract_match_2345() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [2, 3, 4, 5];
        let result = extract_matches(&a, &b, 4);
        assert_eq!(result, vec![make_match_interval(2, 0, 4)]);
    }

    #[test]
    fn extract_match_45() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [0, 4, 5, 0];
        let result = extract_matches(&a, &b, 1);
        assert_eq!(
            result,
            vec![
                make_match_interval(0, 0, 1), // 0.
                make_match_interval(4, 1, 2), // 4 5.
                make_match_interval(0, 3, 1), // 0.
            ]
        );
    }

    #[test]
    fn extract_match_123_567() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [5, 6, 7, 9, 9, 1, 2, 3];
        let result = extract_matches(&a, &b, 1);
        assert_eq!(
            result,
            vec![
                make_match_interval(5, 0, 3), // 5 6 7.
                make_match_interval(1, 5, 3), // 1 2 3.
            ]
        );
    }

    #[test]
    fn delta_123_567() {
        use Compression::*;
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [5, 6, 7, 9, 9, 1, 2, 3];
        let result = delta(&a, &b, 3);
        assert_eq!(result, vec![Match(5, 3), Raw(&[9, 9]), Match(1, 3)]);
    }

    #[test]
    fn delta_no_match() {
        use Compression::*;
        let a = [0, 1, 2, 3, 4, 5];
        let b = [9, 9, 9, 9, 9, 9];
        let result = delta(&a, &b, 3);
        assert_eq!(result, vec![Raw(&b[..])]);
    }

    #[test]
    fn delta_ends_with_raw() {
        use Compression::*;
        let a = [0, 1, 2, 3, 4, 5];
        let b = [9, 9, 9, 3, 4, 5, 9];
        let result = delta(&a, &b, 3);
        assert_eq!(result, vec![Raw(&[9, 9, 9]), Match(3, 3), Raw(&[9])]);
    }

    #[test]
    fn restore_123_567() {
        let a = [0, 1, 2, 3, 4, 5, 6, 7];
        let b = [5, 6, 7, 9, 9, 1, 2, 3];
        let delta = delta(&a, &b, 3);
        let result = restore(&a, &delta);
        assert_eq!(result, vec![&b[0..3], &b[3..5], &b[5..]]);
    }

    #[test]
    fn match_interval_new() {
        let a = [0, 1, 2, 3, 4, 5];
        let b = [2, 3, 4];
        let result = MatchInterval::new(&a, &b, 3, 1);
        assert_eq!(result, make_match_interval(2, 0, 3));
    }

    #[test]
    fn match_interval_remove_overlap_partial() {
        // m1 : |--------|
        // m2 :      |--------|
        // m2':           |---|
        let m1 = make_match_interval(0, 0, 10);
        let mut m2 = make_match_interval(3, 5, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, make_match_interval(9, 11, 4));
    }

    #[test]
    fn match_interval_remove_overlap_all() {
        // m1 : |--------|
        // m2 :   |------|
        // m2':   ||
        let m1 = make_match_interval(0, 0, 10);
        let mut m2 = make_match_interval(3, 5, 5);
        m2.remove_overlap(&m1);
        assert_eq!(m2, make_match_interval(3, 5, 0));
    }

    #[test]
    fn match_interval_remove_overlap_same() {
        // m1 : |--------|
        // m2 : |--------|
        // m2': ||
        let m1 = make_match_interval(0, 0, 10);
        let mut m2 = make_match_interval(0, 0, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, make_match_interval(0, 0, 0));
    }

    #[test]
    fn match_interval_remove_overlap_empty() {
        // m1 : ||
        // m2 : |--------|
        // m2': |--------|
        let m1 = MatchInterval::empty();
        let mut m2 = make_match_interval(0, 0, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, make_match_interval(0, 0, 10));
    }

    #[test]
    fn match_interval_remove_overlap_none() {
        // m1 : |--------|
        // m2 :           |--------|
        // m2':           |--------|
        let m1 = make_match_interval(0, 0, 10);
        let mut m2 = make_match_interval(3, 11, 10);
        m2.remove_overlap(&m1);
        assert_eq!(m2, make_match_interval(3, 11, 10));
    }

    #[test]
    fn modpow_31_41() {
        let result = modpow(31, 41);
        assert_eq!(result, 411956758);
    }

    #[test]
    fn rolling_hash_0101x() {
        let mut hashes = RollingHash::new(&[0, 1, 0, 1], 3);
        assert_eq!(hashes.next(), Some((10201, 0)));
        assert_eq!(hashes.next(), Some((20102, 1)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_010101() {
        let mut hashes = RollingHash::new(&[0, 1, 0, 1, 0, 1], 3);
        assert_eq!(hashes.next(), Some((10201, 0)));
        assert_eq!(hashes.next(), Some((20102, 1)));
        assert_eq!(hashes.next(), Some((10201, 2)));
        assert_eq!(hashes.next(), Some((20102, 3)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_abcdefg() {
        let mut hashes = RollingHash::new("abcdefg".as_ref(), 4).step_by(2);
        let expected = [99000101, 100010202, 101020303, 102030404];
        assert_eq!(hashes.next(), Some((expected[0], 0)));
        assert_eq!(hashes.next(), Some((expected[2], 2)));
        assert_eq!(hashes.next(), None);
    }

    #[test]
    fn rolling_hash_exceeds_mod() {
        let data = vec![255u8; 20];
        let mut hashes = RollingHash::new(&data, 10).step_by(3);
        assert_eq!(hashes.next(), Some((757588431, 0)));
        assert_eq!(hashes.next(), Some((757588431, 3)));
        assert_eq!(hashes.next(), Some((757588431, 6)));
        assert_eq!(hashes.next(), Some((757588431, 9)));
        assert_eq!(hashes.next(), None);
    }
}
