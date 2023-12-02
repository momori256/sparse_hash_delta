# Sparse Hash Delta

This is a Rust implementation of sparse hash delta method, an algorithm designed for obtaining the differences between two files, especially when dealing with large files.

## Overview

The algorithm utilizes a hash table for efficiently finding matching intervals between two files.

Let's illustrate finding a matching interval between `phantom` and `elephant`. The expected result is `phant`, which spans from index 0 to 4 in `phantom` and from index 3 to 7 in `elephant`.

Firstly, a hash table is created from `phantom` with a length of 3.

```
 0 1 2 3 4 5 6
┌─┬─┬─┬─┬─┬─┬─┐
│p│h│a│n│t│o│m│
└─┴─┴─┴─┴─┴─┴─┘
├─────┤ 0x116776
  ├─────┤ 0x102C47
    ├─────┤ 0x0F1FF1
      ├─────┤ 0x111E14
        ├─────┤ 0x12067E
```

The resulting hash table appears as follows:

```
   hash   │ index
 ─────────┼───────
 0x116776 │   0
 0x102C47 │   1
 0x0F1FF1 │   2
 0x111E14 │   3
 0x12067E │   4
```

Next, we iterate through `elephant` and check if each hash is included in the hash table.

```
 0 1 2 3 4 5 6 7
┌─┬─┬─┬─┬─┬─┬─┬─┐
│e│l│e│p│h│a│n│t│
└─┴─┴─┴─┴─┴─┴─┴─┘
├─────┤ 0x0FBB5A
  ├─────┤ 0x10CA19
    ├─────┤ 0x0FBCED
      ├─────┤ 0x116776
        ├─────┤ 0x102C47
          ├─────┤ 0x10F1FF
```

Since `0x116776` is present in both words, we can conclude that `phantom[0] == elephat[3]`. That gives us the desired result.

```
 0 1 2 3 4 5 6 7
 ■───────>
┌─┬─┬─┬─┬─┬─┬─┬─┐
│p│h│a│n│t│o│m│ │
└─┴─┴─┴─┴─┴─┴─┴─┘
       ■───────>
┌─┬─┬─┬─┬─┬─┬─┬─┐
│e│l│e│p│h│a│n│t│
└─┴─┴─┴─┴─┴─┴─┴─┘
```

While this approach appears effective, a potential issue arises with the memory consumption of the hash table, particularly for larger arrays. Applying this method to a substantial array, such as 200MB, results in a hash table with approximately 200MB keys, leading to performance concerns.

Here, the sparse hash delta method proves valuable. Instead of storing all hashes, it stores parts of hashes to reduce memory consumption significantly. The basic concept involves adjusting the hash length based on the desired match length.

```
 0 1 2 3 4 5 6 7 8 9 A B C
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│c│l│a│r│i│f│i│c│a│t│i│o│n│
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
├───────┤ 0x0606A98B
        ├───────┤ 0x06614FBC
                ├───────┤ 0x5E960A8
```

This process results in a hash table with a reduced number of stored hashes.

```
    hash    │ index
 ───────────┼───────
 0x0606A98B │   0
 0x06614FBC │   1
 0x05E960A8 │   2

```

Next, we traverse `declaration` from start to finish.

```
 0 1 2 3 4 5 6 7 8 9 A
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│d│e│c│l│a│r│a│t│i│o│n│
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
├───────┤ 0x0614DB1D
  ├───────┤ 0x0623D2B6
  ...
            ├───────┤ 0x05E960A8
              ├───────┤ 0x0709A00F
```

`0x05E960A8` is present in the hash table created from `clarification`, allowing the detection of the matching interval containing `ation`.

However, this approach may not always work. In scenarios where the hash table does not include matching hashes, certain matching intervals may remain undetected.

```
 0 1 2 3 4 5 6 7 8 9 A B C
├───────┼───────┼───────┤
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │ │ │■│■│■│■│■│■│ │ │
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │■│■│■│■│■│■│ │ │ │ │
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
```

In the best-case scenario, short overlapping intervals like [4, 7] can be identified when the second hash contains it. Even in the worst-case scenario, intervals like [5, B] can be identified since the third hash contains [8, B].

```
 0 1 2 3 4 5 6 7 8 9 A B C
├───────┼───────┼───────┤
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │ │■│■│■│■│ │ │ │ │ │
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
┌─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │ │ │■│■│■│■│■│■│■│ │
└─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┴─┘
```

Generally, the relationship between hash length (`Lh`) and match length (`Lm`) is as follows:

```
Lh + (Lh - 1) <= Lm
Lh <= (Lm + 1) / 2
```

Therefore, if you wnat to find all matches with a length greator than or equal to `Lm`, setting the hash length to `(Lm + 1) / 2` ensures that all matches are detected.

This significantly reduces memory consumption, allowing you to adjust the number of hashes stored in a hash table by defining the minimum length of matches that should be detected.

## Implementation

### Creating a hash table

The algorithm uses a [rolling hash](https://en.wikipedia.org/wiki/Rolling_hash) for creating a hash table. It is a suitable choice in this context.

### Remove overlap between intervals

Overlap between detected match intervals should be resolved. For example, [1, 4] and [3, 7] are overlapping, so they can be resolved to [1, 4] and [5, 7].

```
 0 1 2 3 4 5 6 7
┌─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │ │ │ │ │ │
└─┴─┴─┴─┴─┴─┴─┴─┘
  ├───────┤
      ├─────────┤
```

```
 0 1 2 3 4 5 6 7
┌─┬─┬─┬─┬─┬─┬─┬─┐
│ │ │ │ │ │ │ │ │
└─┴─┴─┴─┴─┴─┴─┴─┘
  ├───────┤
          ├─────┤
```

## Example

The `benchmark.rs` shows how to use this library. It reads two files, obtains the differences between them, and restore the original file. The matching ratio and elapsed time are displayed.

```sh
cargo run -release --example=benchmark // This uses a.txt and b.txt in example directory.
cargo run -release --example=benchmark -- a.dat b.dat // File names can be passed.
```
