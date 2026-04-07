# Sigil Test Runtime Specification

**Version:** 0.1.0-draft
**Status:** Draft
**Author:** Conclave Session
**Date:** 2026-02-11

## Overview

This specification defines the requirements for Sigil's test runtime to support the three-tier testing architecture used in Orpheus UI. The current parser (v0.4.0-rc.4) validates syntax but lacks runtime support for in-source test modules.

## Problem Statement

### Current Limitations

1. **Test Discovery**: `sigil test` only searches `tests/*.sigil`, ignoring in-source test modules (`src-sg/tests/*.sg`)

2. **Module Pattern**: Orpheus uses Rust-style in-source tests:
   ```sigil
   scroll tests {
       invoke super·*;

       //@ rune: test
       rite test_something() { ... }
   }
   ```

3. **Array Indexing**: Parser rejects `array[variable]` with "index must be integer" error

4. **Attribute Syntax**: Mixed messaging - parser rejects `#[test]` but `sigil test` help shows it

## Requirements

### R1: In-Source Test Module Discovery

**Priority:** Critical

The test runner MUST discover test functions in:
- Top-level `tests/*.sigil` files (current behavior)
- In-source `scroll tests { }` modules within any `.sg` file
- Nested test scrolls: `scroll foo { scroll tests { } }`

**Discovery Algorithm:**
```
1. Parse all .sg files in src-sg/ recursively
2. For each file, find `scroll tests { }` blocks
3. Within test scrolls, find functions with `//@ rune: test` attribute
4. Build test index: (file, module_path, test_name)
```

### R2: Test Attribute Support

**Priority:** Critical

The runtime MUST support these test attributes:

| Attribute | Purpose | Example |
|-----------|---------|---------|
| `//@ rune: test` | Mark function as test | `//@ rune: test` |
| `//@ rune: should_panic` | Expect panic | `//@ rune: should_panic` |
| `//@ rune: should_panic(expected = "...")` | Expect specific panic | `//@ rune: should_panic(expected = "dB assertion")` |
| `//@ rune: ignore` | Skip test | `//@ rune: ignore` |
| `//@ rune: ignore(reason = "...")` | Skip with reason | `//@ rune: ignore(reason = "parser limitation")` |
| `//@ rune: track_caller` | Enhanced panic location | `//@ rune: track_caller` |

### R3: Test Execution Model

**Priority:** Critical

#### 3.1 Isolation

Each test MUST run in isolation:
- Fresh state for each test function
- No shared mutable state between tests
- Deterministic execution order (alphabetical by default)

#### 3.2 Parallel Execution

Tests SHOULD support parallel execution:
```bash
sigil test --jobs 4        # Run 4 tests in parallel
sigil test --jobs auto     # Auto-detect CPU count
```

#### 3.3 Output Modes

```bash
sigil test                 # Summary output
sigil test --verbose       # Full output including passing tests
sigil test --quiet         # Only failures
sigil test --format json   # Machine-readable output
```

### R4: Test Filtering

**Priority:** High

```bash
# By name pattern
sigil test mixer           # Tests containing "mixer"
sigil test "test_solo_*"   # Glob pattern

# By module
sigil test --module mixer_integration

# By file
sigil test --file src-sg/tests/mixer_integration.sg

# By attribute
sigil test --include-ignored   # Run ignored tests
```

### R5: Assertion Library

**Priority:** High

Built-in assertions the runtime MUST provide:

```sigil
// Equality
assert_eq!(actual, expected)
assert_ne!(actual, expected)

// Boolean
assert!(condition)
assert!(condition, "message")

// Approximate equality (for floats)
assert_approx_eq!(actual, expected, tolerance)

// Collections
assert!(collection.contains(&item))
assert!(collection.is_empty())

// Panic capture (for testing error conditions)
assert_panics!(|| { dangerous_code() })
assert_panics!(|| { code() }, "expected message")
```

### R6: Parser Enhancements

**Priority:** Critical

#### 6.1 Variable Array Indexing

The parser MUST support array indexing with runtime values:

```sigil
// Currently rejected - MUST work:
≔ arr = [1, 2, 3, 4, 5];
≔ idx: usize = 2;
≔ value = arr[idx];        // Should yield 3

// Also support:
∀ i ∈ 0..arr.len() {
    print(arr[i]);          // Must work
}
```

#### 6.2 Range Expressions in Array Context

```sigil
≔ slice = arr[1..3];        // Slice from index 1 to 3
≔ rest = arr[2..];          // From index 2 to end
≔ start = arr[..3];         // From start to index 3
```

### R7: Test Fixtures Support

**Priority:** Medium

Support for setup/teardown:

```sigil
scroll tests {
    //@ rune: before_each
    rite setup() {
        // Runs before each test
    }

    //@ rune: after_each
    rite teardown() {
        // Runs after each test
    }

    //@ rune: before_all
    rite setup_suite() {
        // Runs once before all tests in this scroll
    }

    //@ rune: after_all
    rite teardown_suite() {
        // Runs once after all tests in this scroll
    }
}
```

### R8: Test Coverage

**Priority:** Low (Future)

```bash
sigil test --coverage              # Generate coverage report
sigil test --coverage-html ./cov   # HTML report
sigil test --coverage-threshold 80 # Fail if <80% coverage
```

## Non-Requirements

These are explicitly out of scope:

1. **Mocking framework** - Use manual test doubles
2. **Property-based testing** - Use external library (future)
3. **Benchmark framework** - Separate `sigil bench` command (future)
4. **Snapshot testing** - Not needed for this use case

## Test Output Format

### Summary Output (Default)

```
Running 127 tests from 8 modules...

✓ assertions (14 tests) ..................... 0.02s
✓ audio_controls_integration (18 tests) ..... 0.05s
✓ audio_controls_e2e (15 tests) ............. 0.08s
✓ mixer_integration (20 tests) .............. 0.04s
✓ mixer_e2e (32 tests) ...................... 0.12s
✓ test_harness (10 tests) ................... 0.01s
✓ test_fixtures (12 tests) .................. 0.02s
✓ mod (6 tests) ............................. 0.01s

Tests: 127 passed, 0 failed, 0 ignored
Time:  0.35s
```

### Failure Output

```
FAILED: mixer_integration::solo_behavior::test_single_solo_isolates_channel

  assertion failed: `assert_eq!(soloed.len(), 1)`
    left:  `2`
    right: `1`

  at src-sg/tests/mixer_integration.sg:35:9

  Note: Multiple channels were soloed when only one was expected.
        This may indicate a state leak between tests.

────────────────────────────────────────────────────────────

Tests: 126 passed, 1 failed, 0 ignored
Time:  0.34s
```

### JSON Output

```json
{
  "summary": {
    "total": 127,
    "passed": 126,
    "failed": 1,
    "ignored": 0,
    "duration_ms": 340
  },
  "modules": [
    {
      "name": "mixer_integration",
      "tests": [
        {
          "name": "test_single_solo_isolates_channel",
          "status": "failed",
          "duration_ms": 2,
          "failure": {
            "message": "assertion failed: `assert_eq!(soloed.len(), 1)`",
            "location": "src-sg/tests/mixer_integration.sg:35:9",
            "left": "2",
            "right": "1"
          }
        }
      ]
    }
  ]
}
```

## Implementation Priority

### Phase 1: Core Test Runner (Required for Orpheus)

1. In-source test discovery (R1)
2. Basic test attributes: `test`, `should_panic`, `ignore` (R2)
3. Test execution with isolation (R3.1)
4. Variable array indexing (R6.1)

### Phase 2: Developer Experience

1. Test filtering (R4)
2. Parallel execution (R3.2)
3. Verbose/quiet/JSON output (R3.3)
4. Range expressions (R6.2)

### Phase 3: Advanced Features

1. Fixtures (R7)
2. Coverage (R8)
3. Enhanced assertions (R5)

## Migration Path

For projects using current `tests/*.sigil` pattern:

1. Both patterns MUST work simultaneously
2. In-source tests take precedence if duplicate names exist
3. Warning if test function exists in both locations

## Acceptance Criteria

The specification is complete when:

1. `sigil test` discovers and runs all 127 tests in orpheus-ui
2. All test attributes work as documented
3. `test_large_project()` function works with variable array indexing
4. Test output matches specified format
5. Exit code is 0 for all pass, non-zero for any failure

## References

- [Rust Test Framework](https://doc.rust-lang.org/book/ch11-00-testing.html) - Inspiration for in-source test pattern
- [Orpheus UI TDD Roadmap](../TDD-ROADMAP.md) - Test architecture context
- [Agent-TDD Methodology](../../../../docs/methodologies/AGENT-TDD.md) - Testing philosophy
