# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

For comprehensive instructions, see [AGENTS.md](./AGENTS.md).

## Quick Reference

The repository uses a Nix-based pre-commit hook that automatically runs all validation checks on commit:

```bash
# All validation happens automatically when you commit
git add . && git commit -m "feat: your message"
```

**CRITICAL**: NEVER use `git commit --no-verify` or bypass the pre-commit hook. The hook ensures all commits pass CI checks.

For the complete development workflow, code architecture, style guidelines, and all other details, refer to [AGENTS.md](./AGENTS.md).
