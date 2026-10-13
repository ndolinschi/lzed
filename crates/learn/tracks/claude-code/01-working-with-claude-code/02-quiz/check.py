import re
answers = dict(re.findall(r"^\s*(Q\d)\s*:\s*([a-cA-C?])", user_source, re.M))
expected = dict(re.findall(r"^\s*(Q\d)\s*:\s*([a-c])", solution_source, re.M))
for q, a in sorted(expected.items()):
    given = answers.get(q, "?").lower()
    check(given == a, q, f"{q}: " + ("answer it by replacing `?` with a letter." if given == "?" else "not quite — reread the question and try again."))
