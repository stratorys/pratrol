---
title: Reduce False Positive Alerts in PR Triage
description: Keep signal quality high by handling generated files, vendored code, and mechanical refactors explicitly.
date: March 1, 2026
author: Pratrol Team
category: Operations
---

False positives usually come from predictable sources:

- Generated files with large diffs.
- Vendored dependencies changing in bulk.
- Mechanical refactors with high line churn.

Create explicit handling rules for those categories early.

When your triage signal feels fair, reviewers trust it and use it daily.
