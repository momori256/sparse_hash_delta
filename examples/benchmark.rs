use sparse_hash_delta::*;

fn main() -> std::io::Result<()> {
    let now = std::time::Instant::now();

    let file_a = std::env::args().nth(1).unwrap_or("a.txt".to_string());
    let file_b = std::env::args().nth(2).unwrap_or("b.txt".to_string());
    let a = std::fs::read(file_a)?;
    let b = std::fs::read(file_b)?;

    // The bigger the file size is, the more sparse the hash interval should be.
    let hash_len = std::cmp::max(10, b.len() / 1000);

    // d = b - a.
    let d = delta(&a, &b, hash_len);

    // Calculate matching ratio.
    let matching_sum: usize = d
        .iter()
        .map(|m| {
            return match m {
                Compression::Match(_, len) => *len,
                Compression::Raw(_) => 0,
            };
        })
        .sum();
    println!(
        "matching ratio: {}",
        (matching_sum as f64) / (b.len() as f64),
    );

    // r = a + d.
    let r = restore(&a, &d);
    let len = r.iter().map(|x| x.len()).sum();
    assert_eq!(b.len(), len);

    println!("{} ms", now.elapsed().as_millis());
    Ok(())
}
