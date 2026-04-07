# Lessons Learned

Organizational memory for the Orpheus project. Document mistakes, discoveries, and successful patterns here so future agents don't repeat failures or miss proven approaches.

## Format

Each entry should follow this structure:

```
## [Date] - [Session/Feature Name]

### Context
What were we trying to do?

### What Happened
What went wrong or right?

### Root Cause
Why did this happen?

### Lesson
What should future agents know?

### Prevention
How do we avoid this in future?
```

---

## Entries

*Add new entries above this line, newest first.*

---

## 2026-02-11 - Initial Infrastructure Setup

### Context
Setting up Daemoniorum methodology infrastructure for the Orpheus project as part of migration to pure Sigil.

### What Happened
Created CONCLAVE.sigil, LESSONS-LEARNED.md, docs structure, and updated CLAUDE.md with methodology references.

### Root Cause
Project was extracted from monorepo without Daemoniorum's current best practice infrastructure.

### Lesson
Extracted projects need infrastructure updates to align with current Daemoniorum practices before major work begins.

### Prevention
When extracting projects from monorepo, check against current methodology docs and add missing infrastructure before starting feature work.
