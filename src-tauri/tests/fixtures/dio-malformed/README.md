# Synthetic malformed data-I/O corpus

Every `malformed-*` file in this directory is deliberately broken and contains synthetic names
only. `truncation-seed.las` is the valid source used to generate the 100 byte-offset truncations in
the SB-DIO-061 contract test. No client delivery or vendor-owned fixture belongs here.

The normal Rust test gate runs the corpus through the registered parser and Intake readers. Adding
a public `parse_*`, `read_*`, `extract_*`, `sniff_*`, or `probe*` file reader without registering it
causes that test to fail.
