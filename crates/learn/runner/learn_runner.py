#!/usr/bin/env python3
"""LearnZed exercise runner.

Usage:
  learn_runner.py run   --setup S --script U
  learn_runner.py check --setup S --script U --solution X --check C

`run` executes setup + user script with normal stdout (for the terminal).
`check` prints exactly one JSON object on stdout:
  {"passed": bool, "checks": [{"name", "passed", "message"}], "stdout", "stderr"}
"""
import argparse
import ast
import contextlib
import io
import json
import re
import sys
import traceback


def _read(path):
    if not path:
        return ""
    with open(path, encoding="utf-8") as f:
        return f.read()


def _exec(source, filename, namespace):
    """Execute source in namespace, capturing stdout/stderr. Returns (out, err, exc_text)."""
    out, err = io.StringIO(), io.StringIO()
    exc_text = None
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        try:
            exec(compile(source, filename, "exec"), namespace)
        except BaseException:  # noqa: BLE001 - report everything to the learner
            etype, evalue, tb = sys.exc_info()
            # Drop the runner's own frame.
            exc_text = "".join(traceback.format_exception(etype, evalue, tb.tb_next))
    return out.getvalue(), err.getvalue(), exc_text


def _equal(a, b):
    try:
        import numpy as np  # noqa: F401

        if isinstance(a, np.ndarray) or isinstance(b, np.ndarray):
            return isinstance(a, np.ndarray) and isinstance(b, np.ndarray) and a.shape == b.shape and bool(np.array_equal(a, b))
    except ImportError:
        pass
    try:
        if isinstance(a, float) or isinstance(b, float):
            return abs(float(a) - float(b)) <= 1e-9 * max(1.0, abs(float(b)))
        return bool(a == b)
    except Exception:  # noqa: BLE001
        return False


def _called_functions(source):
    names = set()
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return names
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            f = node.func
            if isinstance(f, ast.Name):
                names.add(f.id)
            elif isinstance(f, ast.Attribute):
                names.add(f.attr)
                parts = []
                while isinstance(f, ast.Attribute):
                    parts.append(f.attr)
                    f = f.value
                if isinstance(f, ast.Name):
                    parts.append(f.id)
                    names.add(".".join(reversed(parts)))
    return names


def strip_comments(src):
    """Remove // line comments and /* */ block comments (C-like languages)."""
    src = re.sub(r"/\*.*?\*/", "", src, flags=re.S)
    return re.sub(r"//[^\n]*", "", src)


def balanced(src):
    pairs = {")": "(", "]": "[", "}": "{"}
    stack = []
    for ch in re.sub(r'"(?:\\.|[^"\\])*"', '""', src):
        if ch in "([{":
            stack.append(ch)
        elif ch in pairs:
            if not stack or stack.pop() != pairs[ch]:
                return False
    return not stack


def markdown_sections(src):
    """Map lowercase `## Heading` names to their body text."""
    sections, current = {}, None
    for line in src.splitlines():
        m = re.match(r"^##\s+(.*?)\s*$", line)
        if m:
            current = m.group(1).lower()
            sections[current] = ""
        elif current is not None:
            sections[current] += line + "\n"
    return sections


def run(args):
    if args.language != "python":
        print(f"This is a {args.language} exercise that is checked statically. Press Submit to check it.")
        return 0
    ns = {"__name__": "__main__"}
    exec(compile(_read(args.setup), "setup.py", "exec"), ns)
    sys.argv = [args.script]
    try:
        exec(compile(_read(args.script), args.script, "exec"), ns)
    except SystemExit:
        pass
    except BaseException:  # noqa: BLE001
        etype, evalue, tb = sys.exc_info()
        sys.stderr.write("".join(traceback.format_exception(etype, evalue, tb.tb_next)))
        return 1
    return 0


def check(args):
    setup_src = _read(args.setup)
    user_src = _read(args.script)
    checks = []
    if args.language != "python":
        return check_static(args, user_src)

    user_ns = {"__name__": "__main__"}
    _, _, setup_exc = _exec(setup_src, "setup.py", user_ns)
    if setup_exc:
        return {"passed": False, "checks": [{"name": "setup", "passed": False, "message": "Exercise setup failed:\n" + setup_exc}], "stdout": "", "stderr": setup_exc}
    user_out, user_err, user_exc = _exec(user_src, "script.py", user_ns)

    sol_ns = {"__name__": "__main__"}
    _exec(setup_src, "setup.py", sol_ns)
    sol_out, _, sol_exc = _exec(_read(args.solution), "solution.py", sol_ns)

    if user_exc:
        last = user_exc.strip().splitlines()[-1] if user_exc.strip() else "error"
        checks.append({"name": "script runs", "passed": False, "message": "Your code raised an error: " + last})
        return {"passed": False, "checks": checks, "stdout": user_out, "stderr": user_err + user_exc}

    def add(name, ok, message):
        checks.append({"name": name, "passed": bool(ok), "message": "" if ok else message})
        return bool(ok)

    def check_object(name, message=None):
        return add(f"object `{name}`", name in user_ns, message or f"Did you define the variable `{name}`?")

    def check_equal(name, message=None):
        if name not in user_ns:
            return add(f"`{name}` is correct", False, f"Did you define the variable `{name}`?")
        return add(f"`{name}` is correct", _equal(user_ns[name], sol_ns.get(name)), message or f"The value of `{name}` is not correct.")

    def check_output_contains(text, message=None):
        return add(f"output contains {text!r}", str(text) in user_out, message or f"Expected the output to contain `{text}`.")

    def check_output_matches_solution(message=None):
        expected = [l.strip() for l in sol_out.splitlines() if l.strip()]
        got = user_out
        return add("output matches", all(l in got for l in expected), message or "Your output doesn't match the expected output.")

    def check_function_called(name, message=None):
        return add(f"called `{name}()`", name in _called_functions(user_src), message or f"Did you call `{name}()`?")

    def check(condition, name="custom check", message="Check failed."):
        return add(name, condition, message)

    check_ns = {
        "check_object": check_object,
        "check_equal": check_equal,
        "check_output_contains": check_output_contains,
        "check_output_matches_solution": check_output_matches_solution,
        "check_function_called": check_function_called,
        "check": check,
        "user": user_ns,
        "solution": sol_ns,
        "user_output": user_out,
        "solution_output": sol_out,
    }
    _, _, check_exc = _exec(_read(args.check), "check.py", check_ns)
    if check_exc:
        checks.append({"name": "checker", "passed": False, "message": "The exercise checker crashed:\n" + check_exc})
    if sol_exc:
        checks.append({"name": "solution", "passed": False, "message": "The reference solution failed (track bug):\n" + sol_exc})
    passed = bool(checks) and all(c["passed"] for c in checks)
    return {"passed": passed, "checks": checks, "stdout": user_out, "stderr": user_err}


def check_static(args, user_src):
    """Exercises in other languages: check.py inspects the source text."""
    checks = []

    def check(condition, name="custom check", message="Check failed."):
        ok = bool(condition)
        checks.append({"name": name, "passed": ok, "message": "" if ok else message})
        return ok

    ns = {
        "check": check,
        "user_source": user_src,
        "solution_source": _read(args.solution),
        "strip_comments": strip_comments,
        "balanced": balanced,
        "markdown_sections": markdown_sections,
    }
    _, _, check_exc = _exec(_read(args.check), "check.py", ns)
    if check_exc:
        checks.append({"name": "checker", "passed": False, "message": "The exercise checker crashed:\n" + check_exc})
    passed = bool(checks) and all(c["passed"] for c in checks)
    return {"passed": passed, "checks": checks, "stdout": "", "stderr": ""}


def main():
    p = argparse.ArgumentParser()
    p.add_argument("mode", choices=["run", "check"])
    p.add_argument("--setup")
    p.add_argument("--script", required=True)
    p.add_argument("--solution")
    p.add_argument("--check")
    p.add_argument("--language", default="python")
    args = p.parse_args()
    if args.mode == "run":
        sys.exit(run(args))
    result = check(args)
    sys.stdout.write(json.dumps(result))
    sys.stdout.flush()


if __name__ == "__main__":
    main()
