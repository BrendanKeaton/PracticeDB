# PracticeDB

A SQL database engine built from scratch in Rust. This project implements core database internals — a custom SQL parser, page-based storage engine, row encoding, and an interactive CLI — as a hands-on exploration of how relational databases work under the hood.

---

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Architecture](#architecture)
- [Storage Engine](#storage-engine)
- [Supported SQL](#supported-sql)
- [Roadmap](#roadmap)

---

## Features

- **Custom SQL Parser** — Tokenizes and parses SQL commands without relying on a full SQL grammar engine for execution
- **Page-Based Storage** — 4096-byte pages with slotted row layout, stored in `.practice` files
- **Row Encoding/Decoding** — Binary row format with schema-aware type handling
- **Interactive REPL** — Command-line interface for executing queries in real time
- **Catalog System** — Table schemas persisted as JSON metadata for validation and lookups
- **WHERE Filtering** — Conditional row filtering with support for multiple operators and `AND` chains

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)

### Build & Run

```bash
cd PracticeDB
cargo run
```

This launches an interactive prompt where you can issue SQL commands directly.

---

## Usage

### Create a Table

```sql
CREATE profile id:i32:true email:string:false username:string:false
```

Fields are defined as `name:type:is_indexed`. Supported types: `i8`, `i32`, `string`.

### Insert a Row

```sql
INSERT profile 1:user@gmail.com:user
```

Values are colon-separated and must match the schema column order.

### Select Rows

```sql
SELECT * from profile
SELECT id username from profile
```

Supports wildcard (`*`) or specific column names.

### Delete Rows

```sql
DELETE from profile WHERE id = 1
DELETE from profile WHERE id > 5 AND username = admin
```

### Exit

```
exit
```

---

## Architecture

```
src/
├── main.rs              # REPL entry point
├── utils.rs             # Token classification & parse routing
├── core/
│   ├── consts/          # Constants (PAGE_SIZE, etc.)
│   ├── enums/           # Commands, types, operators, filters
│   └── structs/         # QueryObject, Page, TableMetadata, Conditions
├── parsing/
│   ├── mod.rs           # Per-command parse handlers
│   └── utils.rs         # Field & value parsing helpers
└── query/
    ├── mod.rs            # Query execution dispatcher
    ├── create.rs         # CREATE — schema + file creation
    ├── insert.rs         # INSERT — page allocation & row writing
    ├── select.rs         # SELECT — sequential scan & filtering
    ├── delete.rs         # DELETE — conditional row removal
    └── utils.rs          # Row encode/decode, page utilities
```

**Query pipeline:** Input is tokenized and classified into a `QueryObject`, routed through command-specific parsers, then dispatched to the corresponding execution handler which reads/writes `.practice` data files and JSON catalogs.

---

## Storage Engine

### Page Layout (4096 bytes)

Each `.practice` file contains one or more fixed-size pages:

| Bytes | Field             | Description                                              |
| ----- | ----------------- | -------------------------------------------------------- |
| 0     | Page ID           | Offset of the page in the file                           |
| 1     | Row Count         | Number of active rows                                    |
| 2–3   | Current Rows Size | Cumulative size of stored rows (determines insert point) |
| 4–5   | Space Remaining   | Free space available on the page                         |
| 6     | LSN               | Last sequence number (for future WAL support)            |
| 7–8   | Header Size       | Total size of the page header (grows with slots)         |
| 9–10  | Freed Space       | Tracks space from deletions (triggers compaction)        |

After the header, **slots** (4 bytes each: 2 for length, 2 for offset) point to row data stored from the end of the page backward.

### Row Format

| Bytes        | Field          | Description                                |
| ------------ | -------------- | ------------------------------------------ |
| 0            | Header Size    | Size of the row header                     |
| 1            | Num Columns    | Number of columns in the row               |
| 2..2+N       | Column Lengths | One byte per column defining data length   |
| Header end.. | Column Data    | Values stored sequentially in schema order |

### Catalog

Table metadata is stored as JSON in `database/catalogs/<table_name>.json`, containing field names, types, and index flags. This is used for schema validation on inserts and type-aware decoding on reads.

---

## Supported SQL

### Commands

| Command  | Status      | Description                                           |
| -------- | ----------- | ----------------------------------------------------- |
| `CREATE` | Done        | Create a table with typed, optionally indexed columns |
| `INSERT` | Done        | Insert a row with page-based allocation               |
| `SELECT` | Done        | Query rows with column filtering and `WHERE` support  |
| `DELETE` | In Progress | Remove rows matching `WHERE` conditions               |

### Data Types

| Type     | Size            | Description          |
| -------- | --------------- | -------------------- |
| `i8`     | 1 byte          | 8-bit integer        |
| `i32`    | 4 bytes         | 32-bit integer       |
| `string` | Up to 255 bytes | Variable-length text |

### WHERE Operators

`=` `==` `>` `<` `>=` `<=` `!=` `<>`

Multiple conditions can be chained with `AND`.

---

## Roadmap

- [ ] Complete DELETE execution with page compaction
- [ ] Full WHERE clause support across all commands
- [ ] B+Tree indexing on CREATE
- [ ] Write-Ahead Log (WAL) with crash recovery
- [ ] Buffer pool management
- [ ] Default index rules (e.g., first column as primary key)
- [ ] Benchmarking against SQLite

---

## License

See [LICENSE](LICENSE) for details.
