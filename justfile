gen_readme: build
    #!/usr/bin/env python3
    import subprocess

    DEST = "README.md"
    CLI_NAME = "./target/debug/cmux"

    replace = [
        ("<!-- USAGE_START -->\n", "<!-- USAGE_END -->\n", f"{CLI_NAME} -h"),
        ("<!-- USAGE_GEN_START -->\n", "<!-- USAGE_GEN_END -->\n", f"{CLI_NAME} g -h"),
        ("<!-- USAGE_PAR_START -->\n", "<!-- USAGE_PAR_END -->\n", f"{CLI_NAME} p -h"),
    ]

    for start, end, cmd in replace:
        content = ""
        with open(DEST, "r") as f:
            content = f.read()
            old = content[content.find(start) + len(start) : content.rfind(end)]
            new = "```plainstext\n" + subprocess.run(cmd, shell=True, capture_output=True).stdout.decode() + "```\n"
            content = content.replace(old, new)
        with open(DEST, "w") as f:
            f.write(content)

        print(f"Updated {DEST} with {cmd}")


build:
    cargo build

lint:
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

all: build lint test gen_readme