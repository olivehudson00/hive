/* hive - build and test server
 * Copyright (C) 2025 Olive Hudson
 * see LICENCE file for licensing information */

struct Test<'a> {
    name: &'a str,
    provided: &'a str,
    received: &'a str,
    expected: Option<&'a str>,
    info: Vec<&'a str>,
}

pub fn fmt(output: String) -> String {
    let mut lines = output.lines();
    let mut tests: Vec<Test> = Vec::new();

    while let Some(line) = lines.next() {
        let provided = lines.next().unwrap();
        let received = lines.next().unwrap();
        let expected = lines.next().unwrap();

        let info: Vec<'a str> = Vec::new();
        loop {
            let line = lines.next();
            if line.is_none() || line.unwrap().len() == 0 {
                break;
            }
            test.hints.push(line);
        }

        let test = tests.push_mut(Test<'a>{
            name: line,
            provided: provided,
            received: received,
            expected: expected,
            hints: hints,
        });
    }

    let mut string = String::new();
    for test in tests {
        let pass = test.received == test.expected;

        writeln!(string, "<fieldset id='{}'>",
            if pass { "pass" } else { "fail" });
        writeln!(string, "<legend>{}—{}</legend>", test.name,
            if pass { "Passed" } else { "Failed" });
        writeln!(string, "Input: {}<br>", test.provided);
        writeln!(string, "Output: {}<br>", test.received);
        writeln!(string, "<span{}>Expected: {}</span>",
            if pass { "" } else { " id='fail'" }, test.expected);
        for hint in test.hints {
            writeln!(string, "<p>{}</p>", hint);
        }
    }
    return string;
}
