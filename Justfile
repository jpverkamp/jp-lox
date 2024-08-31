build:
    cargo build

test: build test-tokenizer test-parser test-evaluator test-run
update: build update-tokenizer update-parser update-evaluator update-run

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

test-evaluator:
    testit \
        --command "./target/debug/codecrafters-interpreter evaluate -" \
        --files "tests/evaluator/*.lox" \
        --timeout 60 \
        --db tests/evaluator.json

test-run:
    testit \
        --command "./target/debug/codecrafters-interpreter run -" \
        --files "tests/run/*.lox" \
        --timeout 60 \
        --db tests/run.json

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

update-evaluator:
    testit \
        --command "./target/debug/codecrafters-interpreter evaluate -" \
        --files "tests/evaluator/*.lox" \
        --timeout 60 \
        --db tests/evaluator.json \
        --save

update-run:
    testit \
        --command "./target/debug/codecrafters-interpreter run -" \
        --files "tests/run/*.lox" \
        --timeout 60 \
        --db tests/run.json \
        --save