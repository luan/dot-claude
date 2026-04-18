You already have exactly what you need — `src/dates.rs:3` has `humanize_date` which formats a `NaiveDate` as `"March 25, 2026"` using `%B %d, %Y`.

Adding a `format_date` function to `display.rs` that does the same thing would be duplication. I'd recommend either:

1. **Re-export from `display.rs`** if you want it accessible from that module:
   ```rust
   pub use crate::dates::humanize_date as format_date;
   ```

2. **Just use `humanize_date` directly** from `dates` — it already does what you described.

Want me to go with one of these, or is there a different format you had in mind?
