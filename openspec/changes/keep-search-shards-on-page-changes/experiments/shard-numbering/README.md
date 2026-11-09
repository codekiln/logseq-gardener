# Shard numbering experiment

`shard_numbering.py` reads the shards of a site published with search format version 4 and writes them again under other block names and encodings, keeping each shard's key, to compare their total and largest gzip sizes. It compared:

- page numbers in title order (version 4), in a random order, and as route digest slots with 16,384, 32,768 and 65,536 slots;
- each block as a pair of numbers (version 4) or each page's blocks as one group;
- the numbers as a JSON array (version 4) or as one string of base-32 digits.

Run it on a version 4 site of the encode garden:

```sh
python3 shard_numbering.py <site>
```

The results on the encode garden are in `../../verification.md`.
