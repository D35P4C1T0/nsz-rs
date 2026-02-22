# ExecPlan Index

This file tracks all ExecPlans for this repository. It is required by `.agent/PLANS.md`.

## Conventions

- Active plans live in: `.agent/execplans/active/`
- Archived plans live in: `.agent/execplans/archive/`
- Plan filename format: `EP-YYYY-MM-DD__slug.md`
- Plan header fields live inside each plan file and must match the index entry.

## Index entry format (use this consistently)

For each plan, add a single bullet in the appropriate section:

- `EP-YYYY-MM-DD__slug` — `<Title>` — `Status:<DRAFT|ACTIVE|BLOCKED|DONE|ARCHIVED>` — `Created:YYYY-MM-DD` — `Updated:YYYY-MM-DD` — `Path:<repo-relative path>` — `Owner:<UNCONFIRMED|name>` — `Summary:<one line>` — `Links:<optional>`

For archived plans, also include:

- `Archived:YYYY-MM-DD` — `Outcome:<one line>`

Keep entries short, greppable, and consistent.

## Active ExecPlans

- `EP-2026-02-22__nsz-rs-parity` — `NSZ Rust Library 1:1 Parity With Python 4.6.1` — `Status:ACTIVE` — `Created:2026-02-22` — `Updated:2026-02-22` — `Path:.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md` — `Owner:UNCONFIRMED` — `Summary:Implement safe Rust library with byte-identical parity to Python 4.6.1 using parity-first migration` — `Links:docs/plans/2026-02-22-nsz-rs-parity-design.md,docs/plans/2026-02-22-nsz-rs-parity-implementation.md`

## Archived ExecPlans

- (none yet)
