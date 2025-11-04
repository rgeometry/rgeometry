#!/usr/bin/env python3
"""
Extract uncovered code snippets from LCOV coverage data.

This script parses an LCOV file and generates a markdown document containing
all uncovered code snippets with context.
"""

from pathlib import Path
import re
import sys
from typing import Dict, List, Optional, Tuple


def parse_lcov(lcov_file: Path, src_root: Path) -> Dict[str, List[int]]:
    """
    Parse LCOV file and extract uncovered line numbers per file.

    Only includes files that are within src_root to filter out dependencies.

    Returns:
        Dictionary mapping file paths to lists of uncovered line numbers
    """
    uncovered_lines: Dict[str, List[int]] = {}
    current_file: Optional[str] = None
    is_project_file = False

    try:
        with open(lcov_file) as f:
            for line in f:
                line = line.rstrip("\n")

                if line.startswith("SF:"):
                    # Source file line
                    file_path = line[3:]
                    current_file = file_path

                    # Check if this is a project file (relative path, not in /nix/store)
                    is_project_file = "/nix/store" not in file_path

                    if is_project_file and current_file not in uncovered_lines:
                        uncovered_lines[current_file] = []

                elif line.startswith("DA:") and current_file and is_project_file:
                    # Data line: DA:line_number,hit_count
                    match = re.match(r"DA:(\d+),(\d+)", line)
                    if match:
                        line_num = int(match.group(1))
                        hit_count = int(match.group(2))
                        if hit_count == 0:
                            uncovered_lines[current_file].append(line_num)

                elif line == "end_of_record":
                    current_file = None
                    is_project_file = False

    except OSError as e:
        print(f"Error reading LCOV file: {e}", file=sys.stderr)
        sys.exit(1)

    return uncovered_lines


def read_source_file(file_path: Path) -> Optional[List[str]]:
    """
    Read source file and return lines.

    Returns:
        List of lines (with newlines stripped), or None if file not readable
    """
    try:
        with open(file_path, encoding="utf-8", errors="ignore") as f:
            return [line.rstrip("\n") for line in f.readlines()]
    except OSError:
        return None


def get_snippet_context(lines: List[str], line_num: int, context: int = 3) -> Tuple[int, List[str]]:
    """
    Extract a code snippet with context around an uncovered line.

    Args:
        lines: All lines from the source file
        line_num: The uncovered line number (1-indexed)
        context: Number of lines before/after to include

    Returns:
        Tuple of (start_line_number, snippet_lines)
    """
    idx = line_num - 1  # Convert to 0-indexed
    start_idx = max(0, idx - context)
    end_idx = min(len(lines), idx + context + 1)

    start_line = start_idx + 1  # Convert back to 1-indexed
    snippet = lines[start_idx:end_idx]

    return start_line, snippet


def format_line_ranges(line_numbers: List[int]) -> str:
    """
    Format a list of line numbers into a compact range notation.

    Args:
        line_numbers: Sorted list of line numbers

    Returns:
        Formatted string like "213-216, 222-225, 232"

    Examples:
        >>> format_line_ranges([213, 214, 215, 216, 222, 223, 224, 225, 232])
        '213-216, 222-225, 232'
        >>> format_line_ranges([1, 3, 5])
        '1, 3, 5'
    """
    if not line_numbers:
        return ""

    ranges = []
    start = line_numbers[0]
    end = line_numbers[0]

    for num in line_numbers[1:]:
        if num == end + 1:
            # Continue the current range
            end = num
        else:
            # Close the current range and start a new one
            if start == end:
                ranges.append(str(start))
            else:
                ranges.append(f"{start}-{end}")
            start = num
            end = num

    # Add the final range
    if start == end:
        ranges.append(str(start))
    else:
        ranges.append(f"{start}-{end}")

    return ", ".join(ranges)


def generate_report(uncovered_lines: Dict[str, List[int]], src_root: Path, context: int = 3) -> str:
    """
    Generate markdown report of uncovered code snippets.

    Args:
        uncovered_lines: Dictionary mapping file paths to uncovered line numbers
        src_root: Root directory to resolve relative file paths
        context: Number of lines of context around uncovered code

    Returns:
        Markdown formatted report
    """
    report_lines = [
        "# Uncovered Code Snippets\n",
        "This document contains all code snippets that are not covered by tests.\n",
        f"Context: {context} lines before/after each uncovered line.\n",
    ]

    # Filter out files with no uncovered lines and sort for consistent output
    files_with_uncovered = {k: v for k, v in uncovered_lines.items() if v}
    sorted_files = sorted(files_with_uncovered.keys())
    total_uncovered = sum(len(nums) for nums in files_with_uncovered.values())

    report_lines.append("\n## Summary\n")
    report_lines.append(f"- **Total Files**: {len(sorted_files)}\n")
    report_lines.append(f"- **Total Uncovered Lines**: {total_uncovered}\n\n")

    # Process each file
    for file_path_str in sorted_files:
        file_path = Path(file_path_str)

        # Skip if path is absolute but doesn't exist
        if file_path.is_absolute():
            actual_path = file_path
        else:
            # Try to resolve relative to src_root
            actual_path = src_root / file_path
            if not actual_path.exists():
                # Also try as-is
                actual_path = Path(file_path_str)

        # Read source file
        source_lines = read_source_file(actual_path)
        if source_lines is None:
            report_lines.append(f"\n## {file_path_str}\n")
            report_lines.append("⚠️ **Unable to read source file**\n")
            uncovered_nums = sorted(uncovered_lines[file_path_str])
            report_lines.append(f"Uncovered lines: {format_line_ranges(uncovered_nums)}\n")
            continue

        uncovered_nums = sorted(uncovered_lines[file_path_str])

        report_lines.append(f"\n## {file_path_str}\n")
        report_lines.append(f"**Uncovered Lines**: {format_line_ranges(uncovered_nums)}\n\n")

        # Group consecutive uncovered lines to avoid duplicate context
        groups: List[List[int]] = []
        for line_num in uncovered_nums:
            if groups and line_num - groups[-1][-1] <= 2 * context:
                # Extend existing group
                groups[-1].append(line_num)
            else:
                # Create new group
                groups.append([line_num])

        # Generate snippets for each group
        for group_idx, group in enumerate(groups, 1):
            min_line = min(group)
            max_line = max(group)

            # Get combined context for the entire group
            start_idx = max(0, min_line - 1 - context)
            end_idx = min(len(source_lines), max_line + context)
            start_line = start_idx + 1

            snippet = source_lines[start_idx:end_idx]

            end_line = start_line + len(snippet) - 1
            report_lines.append(f"### Snippet {group_idx} (Lines {start_line}-{end_line})\n\n")
            report_lines.append("```rust\n")

            for i, line in enumerate(snippet):
                actual_line_num = start_line + i
                marker = "❌" if actual_line_num in group else "  "
                report_lines.append(f"{marker} {actual_line_num:4d} | {line}\n")

            report_lines.append("```\n\n")

    report_lines.append("---\n")
    report_lines.append("*Report generated by extract-uncovered-snippets.py*\n")

    return "".join(report_lines)


def main():
    if len(sys.argv) < 3:
        print("Usage: extract-uncovered-snippets.py <lcov_file> <output_file> [src_root]")
        sys.exit(1)

    lcov_file = Path(sys.argv[1])
    output_file = Path(sys.argv[2])
    src_root = Path(sys.argv[3]) if len(sys.argv) > 3 else Path.cwd()

    if not lcov_file.exists():
        print(f"Error: LCOV file not found: {lcov_file}", file=sys.stderr)
        sys.exit(1)

    print(f"Parsing LCOV file: {lcov_file}")
    uncovered_lines = parse_lcov(lcov_file, src_root)

    print(f"Found {len(uncovered_lines)} files with uncovered lines")
    print("Generating report...")

    report = generate_report(uncovered_lines, src_root, context=3)

    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, "w") as f:
        f.write(report)

    print(f"Report written to: {output_file}")


if __name__ == "__main__":
    main()
