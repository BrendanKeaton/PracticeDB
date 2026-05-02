# PracticeDB — Where You Left Off

## Snapshot of current state

- `CREATE`, `INSERT`, `SELECT`, `DELETE` all work end-to-end against `.practice` page files
- `DELETE` with `WHERE` (AND / OR + all comparison ops) was just finished — that's the most recent commit
- Storage: 4096-byte slotted pages, sequential scans, JSON catalogs in `database/catalogs/`
- WIP on disk right now: small unstaged edits in `src/core/structs/conditions_object.rs` and `src/query/insert.rs` — review before committing

## Loose ends to clean up first (small, ~1 sitting)

- [ ] Wire `WHERE` into `SELECT` — `parse_sequential` only evaluates conditions when `action == "delete"`; the `else` branch just decodes & prints every row
- [ ] Fix `let _ = return Ok(());` typo at end of [delete.rs:15](PracticeDB/src/query/delete.rs#L15)
- [ ] Reclaim freed row bytes in `delete_row` — currently only the slot is compacted and a "freed space" counter is bumped; actual row bytes stay in the page until a future compaction pass that doesn't exist yet
- [ ] AND/OR precedence in `should_delete_row` is purely left-to-right — `a OR b AND c` evaluates as `(a OR b) AND c`. Decide whether that's "good enough for now" or fix it
- [ ] Guard the 255-row-per-page overflow flagged by the TODO at [insert.rs:47](PracticeDB/src/query/insert.rs#L47)
- [ ] `Page::default()` sets `dirty: true` — fine for `build_new_page`, wrong once you read pages from disk. Will bite you when buffer pool lands

## The Big 3 (in the order that hurts least)

### 1. Buffer Pool
- [ ] `src/core/structs/buffer_pool.rs` is a stub — has `HashMap<u64, Page>` + `capacity`, missing `use Page` import, not in `structs/mod.rs`
- [ ] Pick & implement an eviction policy (LRU is the standard teaching choice)
- [ ] Replace direct `File::open` + `seek`/`read` in `insert.rs`, `select.rs`, `delete.rs`, `utils.rs` with `pool.get_page(table, page_id)` / `pool.flush_page(...)`
- [ ] Honor `pin_count` and `dirty` flags on `Page` (already on the struct — currently unused)
- Doing this *first* makes WAL much easier because every page write goes through one place

### 2. Write-Ahead Log (WAL)
- [ ] Append-only log file (e.g. `database/wal/wal.log`) with records like `{lsn, txn_id, page_id, before, after}`
- [ ] Increment & write the per-page LSN (byte 6 of the page header — already reserved, currently always 0)
- [ ] Force-write WAL record before flushing the dirty page (the actual "write-ahead" rule)
- [ ] Add a transaction wrapper around `INSERT` / `DELETE` so each statement gets a txn_id

### 3. Crash recovery
- [ ] On startup, replay WAL: redo committed txns, undo uncommitted ones (ARIES-lite is fine)
- [ ] Compare each page's LSN against the WAL to skip already-applied records
- [ ] Add a checkpoint mechanism so the WAL doesn't grow unbounded

## After the Big 3 = "done"

Maybe-nice, definitely-skippable:
- B+Tree index using the `is_indexed` flag that's already in the catalog
- Benchmark vs SQLite (already in the README roadmap)
- More types: bool, f64, NULL
