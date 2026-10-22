---
description: 'Rust programming language coding conventions and best practices'
applyTo: 'backend/**/*.rs'
---

# Rust Coding Conventions and Best Practices

Follow idiomatic Rust practices and community standards when writing Rust code. 

These instructions are based on [The Rust Book](https://doc.rust-lang.org/book/), [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), [RFC 430 naming conventions](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md), and the broader Rust community at [users.rust-lang.org](https://users.rust-lang.org).

## General Instructions

- Always prioritize readability, safety, and maintainability.
- Use strong typing and leverage Rust's ownership system for memory safety.
- Break down complex functions into smaller, more manageable functions.
- For algorithm-related code, include explanations of the approach used.
- Write code with good maintainability practices, including comments on why certain design decisions were made.
- Handle errors gracefully using `Result<T, E>` and provide meaningful error messages.
- For external dependencies, mention their usage and purpose in documentation.
- Use consistent naming conventions following [RFC 430](https://github.com/rust-lang/rfcs/blob/master/text/0430-finalizing-naming-conventions.md).
- Write idiomatic, safe, and efficient Rust code that follows the borrow checker's rules.
- Ensure code compiles without warnings.

## Patterns to Follow

- Use modules (`mod`) and public interfaces (`pub`) to encapsulate logic.
- Handle errors properly using `?`, `match`, or `if let`.
- Use `serde` for serialization and `thiserror` or `anyhow` for custom errors.
- Implement traits to abstract services or external dependencies.
- Structure async code using `async/await` and `tokio` or `async-std`.
- Prefer enums over flags and states for type safety.
- Use builders for complex object creation.
- Split binary and library code (`main.rs` vs `lib.rs`) for testability and reuse.
- Use `rayon` for data parallelism and CPU-bound tasks.
- Use iterators instead of index-based loops as they're often faster and safer.
- Use `&str` instead of `String` for function parameters when you don't need ownership.
- Prefer borrowing and zero-copy operations to avoid unnecessary allocations.

### Ownership, Borrowing, and Lifetimes

- Prefer borrowing (`&T`) over cloning unless ownership transfer is necessary.
- Use `&mut T` when you need to modify borrowed data.
- Explicitly annotate lifetimes when the compiler cannot infer them.
- Use `Rc<T>` for single-threaded reference counting and `Arc<T>` for thread-safe reference counting.
- Use `RefCell<T>` for interior mutability in single-threaded contexts and `Mutex<T>` or `RwLock<T>` for multi-threaded contexts.

## Patterns to Avoid
### Type Drift

The architecture is layered: **DB → Domain → API**. Each layer has a **source of truth** for its types:

- **DB layer**: the migration SQL (`migrations/*.sql`)
- **Domain layer**: business invariants defined in `domain/`
- **API layer**: the JSON/OpenAPI schema exposed to clients
- **A numeric cast (`as i32`, `as i64`, `.into()`, `::bigint` in SQL) at a layer boundary is a red flag.** It means two adjacent layers have diverged from their contract. The fix is always to align the type at the source — never to silence the divergence with a cast.
- When adding or modifying a struct that crosses a layer boundary (DB row, domain model, API DTO), trace every field back to its source of truth and match the type exactly.
- A `as T` or explicit type coercion between a struct field and its source signals untracked tech debt — investigate and fix before committing.
- The only legitimate conversions at a boundary are **semantic transformations** explicitly justified by a business rule (e.g., converting a domain type into a serializable representation). Document the reason inline.
- Aggregates and computed expressions (SQL `SUM`, `COUNT`, `AVG`) are an exception: they always return a wider type from the engine (`i64`, `Decimal`). Map them to the wider Rust type; this is intentional, not drift.

### Enum drift — `enum_type::text` in SQL

**Signal**: `::text` cast on a PostgreSQL ENUM in a `query` or `query_as` call (e.g. `status::text as "status!"`).
**Root cause**: The Rust type is `String` but a proper typed enum exists (or should exist) in the DB schema.
**Fix**: Implement a Rust enum deriving `sqlx::Type` (with `#[sqlx(type_name = "...", rename_all = "snake_case")]`) and map it directly. Remove the `::text` cast.
**Exception**: When using the non-macro `query_as::<_, T>()` with a custom struct that pre-dates the typed enum — document why and plan the migration.

### Soft-delete filter drift

**Signal**: A query that reads or joins a table omits the soft-delete filter (`AND active = true` or `AND deleted_at IS NULL`).
**Root cause**: The soft-delete convention for a given table is not applied consistently across all queries touching that table.
**Rule**: Each table uses exactly one soft-delete mechanism — never mix them on the same table. When writing any query against a soft-deletable table, the filter is **mandatory** unless the query is explicitly about archived records (then add a comment).
**Audit**: If `active` column exists → always filter `AND active = true`. If `deleted_at` exists → always filter `AND deleted_at IS NULL`.

### Nullability drift — `Option<T>` vs `NOT NULL`

**Signal**: A `FromRow` struct field is `Option<T>` but the corresponding column is `NOT NULL` in the migration, or the reverse.
**Root cause**: The struct was written without checking the schema, or the migration was updated without updating the struct.
**Fix**: `NOT NULL` in SQL → `T` (not `Option<T>`) in Rust. Nullable → `Option<T>`. Check the migration before adding `Option`.

### Validation drift

**Signal**: A business constraint (min length, range, enum value) is validated in the API DTO (`#[validate(...)]`) but not enforced by a DB constraint (`CHECK`, `NOT NULL`, `REFERENCES`), or validated in the frontend but not in the backend.
**Rule**: Every validation rule has a **primary enforcement layer**:
- Format/length → API DTO (`#[validate]`) + frontend form
- Business invariant (e.g. `shares_total > 0`) → API handler + DB `CHECK` constraint
- Referential integrity → DB `FOREIGN KEY`
A rule that only exists in one layer is latent drift. Add the missing layer or document the exception.

### Soft schema drift (column added to DB, not to struct)

**Signal**: A `cargo check` passes but a runtime query panics or returns unexpected `None` because a new column was added to the migration without being added to the `FromRow` struct (or vice versa).
**Rule**: When writing a new migration that adds, renames, or drops a column on an existing table, immediately update every `FromRow` struct and every `SELECT *` or `SELECT ... RETURNING` query that targets that table.

## Quality Checklist

- Don't use `unwrap()` or `expect()` unless absolutely necessary—prefer proper error handling.
- Avoid panics in library code—return `Result` instead.
- Don't rely on global mutable state—use dependency injection or thread-safe containers.
- Avoid deeply nested logic—refactor with functions or combinators.
- Don't ignore warnings—treat them as errors during CI.
- Avoid `unsafe` unless required and fully documented.
- Don't overuse `clone()`, use borrowing instead of cloning unless ownership transfer is needed.
- Avoid premature `collect()`, keep iterators lazy until you actually need the collection.
- Avoid unnecessary allocations—prefer borrowing and zero-copy operations.

## Code Style and Formatting

- Follow the Rust Style Guide and use `rustfmt` for automatic formatting.
- Keep lines under 100 characters when possible.
- Place function and struct documentation immediately before the item using `///`.
- Use `cargo clippy` to catch common mistakes and enforce best practices.

## Error Handling

- Use `Result<T, E>` for recoverable errors and `panic!` only for unrecoverable errors.
- Prefer `?` operator over `unwrap()` or `expect()` for error propagation.
- Create custom error types using `thiserror` or implement `std::error::Error`.
- Use `Option<T>` for values that may or may not exist.
- Provide meaningful error messages and context.
- Error types should be meaningful and well-behaved (implement standard traits).
- Validate function arguments and return appropriate errors for invalid input.
- Always use custom i18n error codes (e.g., `AppError::validation_code("errors.claims.reimbursement_requires_status")`) instead of generic messages to allow for proper localization and consistency across the codebase.

## API Design Guidelines

### Common Traits Implementation
Eagerly implement common traits where appropriate:
- `Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`, `Default`
- Use standard conversion traits: `From`, `AsRef`, `AsMut`
- Collections should implement `FromIterator` and `Extend`
- Note: `Send` and `Sync` are auto-implemented by the compiler when safe; avoid manual implementation unless using `unsafe` code

### Type Safety and Predictability
- Use newtypes to provide static distinctions
- Arguments should convey meaning through types; prefer specific types over generic `bool` parameters
- Use `Option<T>` appropriately for truly optional values
- Functions with a clear receiver should be methods
- Only smart pointers should implement `Deref` and `DerefMut`

### Future Proofing
- Use sealed traits to protect against downstream implementations
- Structs should have private fields
- Functions should validate their arguments
- All public types must implement `Debug`

## Testing and Documentation

- Write comprehensive unit tests using `#[cfg(test)]` modules and `#[test]` annotations.
- Use test modules alongside the code they test (`mod tests { ... }`).
- Write integration tests in `tests/` directory with descriptive filenames.
- Write clear and concise comments for each function, struct, enum, and complex logic.
- Ensure functions have descriptive names and include comprehensive documentation.
- Document all public APIs with rustdoc (`///` comments) following the [API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `#[doc(hidden)]` to hide implementation details from public documentation.
- Document error conditions, panic scenarios, and safety considerations.
- Examples should use `?` operator, not `unwrap()` or deprecated `try!` macro.

## Project Organization

- Use semantic versioning in `Cargo.toml`.
- Include comprehensive metadata: `description`, `license`, `repository`, `keywords`, `categories`.
- Use feature flags for optional functionality.
- Organize code into modules using `mod.rs` or named files.
- Keep `main.rs` or `lib.rs` minimal - move logic to modules.

## Quality Checklist

Before publishing or reviewing Rust code, ensure:

### Core Requirements
- [ ] **Naming**: Follows RFC 430 naming conventions
- [ ] **Traits**: Implements `Debug`, `Clone`, `PartialEq` where appropriate
- [ ] **Error Handling**: Uses `Result<T, E>` and provides meaningful error types
- [ ] **Documentation**: All public items have rustdoc comments with examples
- [ ] **Testing**: Comprehensive test coverage including edge cases

### Safety and Quality
- [ ] **Safety**: No unnecessary `unsafe` code, proper error handling
- [ ] **Performance**: Efficient use of iterators, minimal allocations
- [ ] **API Design**: Functions are predictable, flexible, and type-safe
- [ ] **Future Proofing**: Private fields in structs, sealed traits where appropriate
- [ ] **Tooling**: Code passes `cargo fmt`, `cargo clippy`, and `cargo test`
