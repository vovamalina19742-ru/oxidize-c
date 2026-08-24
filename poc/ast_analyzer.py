#!/usr/bin/env python3
"""
Oxidize-C: AST & Pattern Classifier Engine (Phase 1)
==================================================
Performs lexical and structural AST analysis of C source files to detect:
1. Self-referential / cyclic graph structures -> Suggests Generational SlotMap
2. Tagged unions / polymorphic structs -> Suggests Rust algebraic enum
3. OS file / socket descriptors -> Suggests I/O Safety RAII (OwnedFd/OwnedSocket)
4. Dynamic memory allocations -> Suggests Box<T> / Vec<T> RAII
"""

import re
import sys
import json
from pathlib import Path
from dataclasses import dataclass, asdict, field
from typing import List, Dict, Optional

# Ensure UTF-8 stdout on Windows
if sys.stdout:
    sys.stdout.reconfigure(encoding='utf-8')

@dataclass
class StructField:
    name: str
    type_str: str
    is_pointer: bool
    is_self_ref: bool

@dataclass
class StructInfo:
    name: str
    fields: List[StructField] = field(default_factory=list)
    is_cyclic: bool = False
    has_union: bool = False
    has_os_descriptor: bool = False
    suggested_rust_pattern: str = "Standard Struct"

@dataclass
class FileAnalysisReport:
    filename: str
    total_lines: int
    structs: List[StructInfo] = field(default_factory=list)
    has_malloc: bool = False
    has_free: bool = False
    has_realloc: bool = False
    has_socket_or_fd: bool = False
    suggested_patterns: List[str] = field(default_factory=list)

class CASTAnalyzer:
    def __init__(self, source_code: str, filename: str = "source.c"):
        self.code = source_code
        self.filename = filename
        self.lines = source_code.splitlines()

    def analyze(self) -> FileAnalysisReport:
        report = FileAnalysisReport(
            filename=self.filename,
            total_lines=len(self.lines)
        )

        # 1. Detect Memory Allocation APIs
        report.has_malloc = bool(re.search(r"\bmalloc\s*\(", self.code))
        report.has_free = bool(re.search(r"\bfree\s*\(", self.code))
        report.has_realloc = bool(re.search(r"\brealloc\s*\(", self.code))

        # 2. Detect OS Descriptors / Sockets / Files
        report.has_socket_or_fd = bool(
            re.search(r"\b(socket|closesocket|close|SOCKET|socket_t|RawFd|OwnedFd|epoll|kqueue|open|read|write)\b", self.code)
        )

        # 3. Extract Structs & Typedefs
        struct_blocks = re.findall(
            r"(?:typedef\s+)?struct\s*([A-Za-z0-9_]*)\s*\{([^}]+)\}\s*([A-Za-z0-9_]*)\s*;",
            self.code
        )

        for tag_name, body, alias_name in struct_blocks:
            s_name = alias_name.strip() or tag_name.strip() or "AnonymousStruct"
            s_info = StructInfo(name=s_name)

            # Check for embedded union
            if "union" in body:
                s_info.has_union = True
                s_info.suggested_rust_pattern = "Algebraic Enum with associated data"

            # Parse fields inside struct
            field_lines = [f.strip() for f in body.split(";") if f.strip()]
            for f_line in field_lines:
                # Clean up comments and qualifiers
                clean_line = re.sub(r"/\*.*?\*/", "", f_line).strip()
                if not clean_line or "union" in clean_line:
                    continue

                # Match type and identifier
                match = re.search(r"([A-Za-z0-9_*\s]+)\s+([*]*)([A-Za-z0-9_]+)$", clean_line)
                if match:
                    type_part = match.group(1).strip()
                    star_part = match.group(2).strip()
                    name_part = match.group(3).strip()

                    is_pointer = "*" in type_part or "*" in star_part
                    is_self_ref = False

                    # Check if field references self struct name or tag
                    if s_name in type_part or (tag_name and tag_name in type_part):
                        if is_pointer:
                            is_self_ref = True
                            s_info.is_cyclic = True

                    # Check for OS socket / fd in field
                    if any(t in type_part or t in name_part for t in ["socket", "sock_fd", "fd", "SOCKET", "socket_t"]):
                        s_info.has_os_descriptor = True

                    s_info.fields.append(StructField(
                        name=name_part,
                        type_str=f"{type_part}{star_part}".strip(),
                        is_pointer=is_pointer,
                        is_self_ref=is_self_ref
                    ))

            # Determine architectural pattern
            if s_info.is_cyclic:
                s_info.suggested_rust_pattern = "Generational Arena (SlotMap with NodeKey)"
            elif s_info.has_os_descriptor:
                s_info.suggested_rust_pattern = "I/O Safety RAII (OwnedFd / OwnedSocket)"
            elif s_info.has_union:
                s_info.suggested_rust_pattern = "Rust Algebraic Enum"
            elif report.has_malloc and report.has_free:
                s_info.suggested_rust_pattern = "RAII Ownership Struct (Vec / Box)"

            report.structs.append(s_info)

        # Aggregate suggested patterns
        patterns = set()
        if report.has_socket_or_fd:
            patterns.add("I/O Safety RAII (std::os::fd::OwnedFd / std::os::windows::io::OwnedSocket)")
        if any(s.is_cyclic for s in report.structs):
            patterns.add("Generational Arena (SlotMap with NodeKey index versioning)")
        if any(s.has_union for s in report.structs):
            patterns.add("Rust Tagged Algebraic Enum")
        if report.has_malloc:
            patterns.add("Automatic RAII Ownership (Vec<T> / Box<T>)")

        report.suggested_patterns = sorted(list(patterns))
        return report

    def format_llm_brief(self, report: FileAnalysisReport) -> str:
        """Formats the AST analysis as a strict directive brief for LLM prompt context."""
        brief = [
            "=== OXIDIZE-C STATIC AST & MEMORY ANALYSIS BRIEF ===",
            f"File: {report.filename} (Lines: {report.total_lines})",
            f"Memory Operations: malloc={report.has_malloc}, free={report.has_free}, realloc={report.has_realloc}",
            f"OS / Socket Descriptors Detected: {report.has_socket_or_fd}",
            "\nSTRUCTURAL CLASSIFICATION:"
        ]
        for s in report.structs:
            cyclic_mark = " [CYCLIC GRAPH]" if s.is_cyclic else ""
            union_mark = " [CONTAINS UNION]" if s.has_union else ""
            os_mark = " [OS DESCRIPTOR]" if s.has_os_descriptor else ""
            brief.append(f"  • struct {s.name}{cyclic_mark}{union_mark}{os_mark}")
            brief.append(f"    └── Recommended Pattern: {s.suggested_rust_pattern}")

        brief.append("\nMANDATORY ARCHITECTURAL DIRECTIVES:")
        for idx, p in enumerate(report.suggested_patterns, 1):
            brief.append(f"  {idx}. {p}")
        brief.append("====================================================")
        return "\n".join(brief)

def main():
    fixtures_dir = Path(__file__).parent / "fixtures"
    if not fixtures_dir.exists():
        print(f"[-] Directory {fixtures_dir} does not exist.")
        return

    print("================================================================")
    print("      OXIDIZE-C: STATIC AST & PATTERN CLASSIFIER TEST SUITE      ")
    print("================================================================\n")

    c_files = sorted(list(fixtures_dir.glob("*.c")))
    sample_input = Path(__file__).parent / "sample_input.c"
    if sample_input.exists():
        c_files.insert(0, sample_input)

    for c_file in c_files:
        code = c_file.read_text(encoding="utf-8")
        analyzer = CASTAnalyzer(code, filename=c_file.name)
        report = analyzer.analyze()
        brief = analyzer.format_llm_brief(report)
        print(brief)
        print("\n" + "-"*64 + "\n")

if __name__ == "__main__":
    main()
