#!/usr/bin/env python3
"""
Add a field to every `StructName { ... }` literal that does not already
have it. Operates line-by-line and tracks brace depth across lines, so
it correctly handles nested braces, comments, strings, and char
literals.

Usage:
    python3 update_structs.py FILE [FILE ...]

For each file, adds `cache: None` to every `Router { ... }` and
`compression: None` to every `Settings { ... }`.
"""
import re
import sys


def line_has_struct_open(line, struct_name):
    """Return True iff `line` opens a `StructName { ... }` literal."""
    # Match struct_name preceded by non-word char (so we don't match
    # `JwtSettings`), followed by optional whitespace, then `{`. Allow
    # the match to be anywhere on the line so patterns like
    # `let settings = Settings {` and `vec![Router {` work.
    m = re.search(r'(?<!\w)' + re.escape(struct_name) + r'\s*\{', line)
    if not m:
        return False
    # Avoid false matches in function signatures like
    # `fn foo() -> Router {`, where the `{` opens the function body
    # rather than a struct literal. If `->` precedes the struct name on
    # the same line, the `{` belongs to the function signature.
    if '->' in line[:m.start()]:
        return False
    return True


def find_closing_line(lines, start_idx):
    """Given lines[start_idx] opens a struct with `{`, return the index
    of the line containing the matching `}`. Tracks nested braces,
    comments, and strings.
    Returns -1 if not found.
    """
    depth = 1  # we already saw the opening `{`
    j = start_idx + 1
    in_string = False
    in_char = False
    in_line_comment = False
    in_block_comment = False
    string_char = None

    while j < len(lines):
        line = lines[j]
        k = 0
        n = len(line)
        while k < n:
            c = line[k]
            c2 = line[k:k + 2]

            if in_line_comment:
                if c == '\n':
                    in_line_comment = False
            elif in_block_comment:
                if c2 == '*/':
                    in_block_comment = False
                    k += 1
            elif in_string:
                if c == '\\':
                    k += 1
                elif c == string_char:
                    in_string = False
            elif in_char:
                if c == '\\':
                    k += 1
                elif c == "'":
                    in_char = False
            else:
                if c2 == '//':
                    in_line_comment = True
                    k += 1
                elif c2 == '/*':
                    in_block_comment = True
                    k += 1
                elif c == '"':
                    in_string = True
                    string_char = '"'
                elif c == "'":
                    in_char = True
                    string_char = "'"
                elif c == '{':
                    depth += 1
                elif c == '}':
                    depth -= 1
                    if depth == 0:
                        return j
            k += 1
        j += 1
    return -1


def update_file(path, struct_name, field_name, field_value):
    with open(path) as f:
        lines = f.read().splitlines(keepends=True)

    out = []
    i = 0
    modified = False
    while i < len(lines):
        line = lines[i]

        if not line_has_struct_open(line, struct_name):
            out.append(line)
            i += 1
            continue

        # Find closing line
        close_idx = find_closing_line(lines, i)
        if close_idx == -1:
            out.append(line)
            i += 1
            continue

        # Build the body of the struct (between open and close)
        body = ''.join(lines[i + 1:close_idx])

        # Already has the field?
        if re.search(r'(?<!\w)' + re.escape(field_name) + r'\s*:', body):
            for k in range(i, close_idx + 1):
                out.append(lines[k])
            i = close_idx + 1
            continue

        # Find field indentation: look at first field line in body
        field_indent = '    '
        for body_line in body.split('\n'):
            m = re.match(r'^(\s*)\w[\w_]*\s*:', body_line)
            if m:
                field_indent = m.group(1)
                break

        # We need to insert `field_name: field_value,` before the closing
        # brace. The simplest way: emit the open line, emit all body lines
        # as-is (they end with `\n`), and emit `field_indent + field_name +
        # ': ' + field_value + ','` followed by the closing line.
        out.append(line)  # the open line `    Router {`
        for k in range(i + 1, close_idx):
            out.append(lines[k])
        out.append(field_indent + field_name + ': ' + field_value + ',\n')
        out.append(lines[close_idx])  # the closing line `    }`
        i = close_idx + 1
        modified = True

    if modified:
        with open(path, 'w') as f:
            f.write(''.join(out))
        return True
    return False


def main():
    files = sys.argv[1:]
    if not files:
        print('usage: update_structs.py FILE [FILE ...]')
        sys.exit(2)

    for path in files:
        if update_file(path, 'Router', 'cache', 'None'):
            print(f'  + cache        in {path}')
        if update_file(path, 'Settings', 'compression', 'None'):
            print(f'  + compression  in {path}')


if __name__ == '__main__':
    main()