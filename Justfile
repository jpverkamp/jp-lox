build:
    cargo build

test: build test-tokenizer test-parser
update: build update-tokenizer update-parser

test-tokenizer:
    testit \
        --command "./target/debug/codecrafters-interpreter tokenize -" \
        --files "tests/tokenizer/*.lox" \
        --timeout 60 \
        --db tests/tokenizer.json

test-parser:
    testit \
        --command "./target/debug/codecrafters-interpreter parse -" \
        --files "tests/parser/*.lox" \
        --timeout 60 \
        --db tests/parser.json

update-tokenizer:
    testit \
        --command "./target/debug/codecrafters-interpreter tokenize -" \
        --files "tests/tokenizer/*.lox" \
        --timeout 60 \
        --db tests/tokenizer.json \
        --save

update-parser:
    testit \
        --command "./target/debug/codecrafters-interpreter parse -" \
        --files "tests/parser/*.lox" \
        --timeout 60 \
        --db tests/parser.json \
        --save