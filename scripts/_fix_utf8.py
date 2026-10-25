import os

SKIP = {"si-audit"}


def fix(p):
    b = open(p, "rb").read()
    b2 = b.replace(b"\xc2\xc2\xa7", b"\xc2\xa7")
    try:
        b2.decode("utf-8")
    except UnicodeDecodeError as e:
        print(p, "STILL INVALID", e)
        return
    if b2 != b:
        open(p, "wb").write(b2)
        print("fixed", p)


for root, _, files in os.walk("crates"):
    for f in files:
        if not f.endswith(".rs"):
            continue
        p = os.path.join(root, f)
        if os.path.basename(root) in SKIP:
            continue
        fix(p)
