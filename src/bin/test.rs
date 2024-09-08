use std::{cell::RefCell, env, fmt::Display, ops::DerefMut, panic, rc::Rc};

use make_a_lisp_rs::{error::MalError, rep, Env};

#[derive(Debug)]
struct Section {
    title: String,
    cases: Vec<Case>,
}

#[derive(Debug, Clone)]
enum ExpectedOutput {
    Literal(String),
    Regex(pcre2::bytes::Regex),
}

impl Display for ExpectedOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedOutput::Literal(literal) => write!(f, ";=>{}", literal),
            ExpectedOutput::Regex(regex) => write!(f, ";/{}", regex.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
struct Case {
    input: Vec<String>,
    expected_output: ExpectedOutput,
    line_number: usize,

    // Meta information
    deferrable: bool,
    soft: bool,
    optional: bool,
}

struct CaseRunConfig {
    skip_deferrable: bool,
    skip_soft: bool,
    skip_optional: bool,
}

impl Case {
    fn run(&self, config: CaseRunConfig, env: Rc<RefCell<Env>>) -> CaseOutput {
        if config.skip_deferrable && self.deferrable {
            return CaseOutput::Skip;
        }
        if config.skip_soft && self.soft {
            return CaseOutput::Skip;
        }
        if config.skip_optional && self.optional {
            return CaseOutput::Skip;
        }

        let result: Result<String, MalError> = (|| {
            let mut output = String::new();
            for line in self.input.iter() {
                output = match rep(line.clone(), env.clone()) {
                    Ok(output) => output,
                    Err(e) => {
                        format!("{}", e)
                    }
                };
            }
            Ok(output)
        })();

        let actual_output = match result {
            Ok(output) => output,
            Err(e) => format!("{}", e),
        };

        let passed = match self.expected_output.clone() {
            ExpectedOutput::Literal(expected_output) => actual_output == expected_output,
            ExpectedOutput::Regex(expected_output) => {
                expected_output.is_match(actual_output.as_bytes()).unwrap()
            }
        };
        if passed {
            CaseOutput::Pass
        } else {
            CaseOutput::Fail(FailingCase {
                input: self.input.clone(),
                expected_output: self.expected_output.clone(),
                actual_output,
                line_number: self.line_number,
                soft: self.soft,
                optional: self.optional,
                deferrable: self.deferrable,
            })
        }
    }
}

struct SectionOutput {
    title: String,
    cases: Vec<CaseOutput>,
}

struct FailingCase {
    input: Vec<String>,
    expected_output: ExpectedOutput,
    actual_output: String,
    line_number: usize,
    soft: bool,
    optional: bool,
    deferrable: bool,
}

enum CaseOutput {
    Pass,
    Skip,
    Fail(FailingCase),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let test_file = args[1].clone();
    println!("Running test file: {}", test_file);

    let input = std::fs::read_to_string(args[1].clone()).unwrap();
    // println!("{}", input);
    let mut lines = input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_number, line)| (line_number + 1, line.to_owned()));

    // meta commands
    let mut deferrable = false;
    let mut soft = false;
    let mut optional = false;

    let mut sections: Vec<Section> = vec![];
    let mut current_section = Section {
        title: "".to_owned(),
        cases: vec![],
    };

    while let Some((line_number, line)) = lines.next() {
        if line.starts_with(";;;") {
            // This is a comment, so we skip it
            continue;
        } else if let Some(stripped_line) = line.strip_prefix(";;") {
            // Begin a new section
            if !current_section.cases.is_empty() {
                sections.push(current_section);
            }
            current_section = Section {
                title: stripped_line.trim().to_owned(),
                cases: vec![],
            };
        } else if let Some(stripped_line) = line.strip_prefix(";>>>") {
            // Define a meta command
            match stripped_line
                .trim()
                .split_once('=')
                .expect("Invalid meta command")
            {
                ("deferrable", value) => {
                    deferrable = match value {
                        "True" => true,
                        "False" => false,
                        _ => panic!("Invalid deferrable value: {}", value),
                    };
                    if deferrable {
                        optional = false;
                    }
                }
                ("soft", value) => {
                    soft = match value {
                        "True" => true,
                        "False" => false,
                        _ => panic!("Invalid soft value: {}", value),
                    };
                }
                ("optional", value) => {
                    optional = match value {
                        "True" => true,
                        "False" => false,
                        _ => panic!("Invalid optional value: {}", value),
                    };
                    if optional {
                        deferrable = false;
                    }
                }
                _ => panic!("Unknown meta command: {}", line),
            }
        } else {
            // Begin a new case
            let mut input = vec![line];
            let case_line_number = line_number;
            // Consume all lines until we hit an expected output line
            for (line_number, line) in lines.by_ref() {
                if line.starts_with(";") {
                    // This is an expected output line
                    let expected_output = if let Some(stripped_line) = line.strip_prefix(";=>") {
                        // This is a literal expected output
                        ExpectedOutput::Literal(stripped_line.to_owned())
                    } else if let Some(stripped_line) = line.strip_prefix(";/") {
                        // This is a regex expected output
                        ExpectedOutput::Regex(pcre2::bytes::Regex::new(stripped_line).unwrap())
                    } else {
                        panic!(
                            "Invalid expected output line on line {}: {}",
                            line_number, line
                        );
                    };
                    // Build the case, push it to the current section, and break from the loop
                    current_section.cases.push(Case {
                        input,
                        expected_output,
                        line_number: case_line_number,
                        deferrable,
                        soft,
                        optional,
                    });
                    break;
                } else {
                    // Push this line to the input
                    input.push(line);
                }
            }
        }
    }

    let env = make_a_lisp_rs::Env::new(None);
    make_a_lisp_rs::load_builtins(env.borrow_mut().deref_mut());

    let section_outputs = sections
        .into_iter()
        .map(|section| SectionOutput {
            title: section.title,
            cases: section
                .cases
                .into_iter()
                .map(|case| {
                    case.run(
                        CaseRunConfig {
                            skip_deferrable: false,
                            skip_soft: false,
                            skip_optional: false,
                        },
                        env.clone(),
                    )
                })
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();

    // Run cases
    let mut num_cases_seen = 0;
    let mut passing_cases = 0;
    let mut soft_fails = 0;
    let mut required_fails = 0;
    let mut skipped_cases = 0;
    for section_output in section_outputs {
        let num_cases = section_output.cases.len();
        let failing_cases = section_output
            .cases
            .iter()
            .enumerate()
            .filter_map(|(i, case)| match case {
                CaseOutput::Fail(case) => Some((i, case)),
                _ => None,
            });
        let num_skipped_cases = section_output
            .cases
            .iter()
            .filter(|case| matches!(case, CaseOutput::Skip))
            .count();
        let num_failing_cases = failing_cases.clone().count();
        let num_soft_failing_cases = failing_cases
            .clone()
            .filter(|(_, case)| case.soft || case.deferrable || case.optional)
            .count();
        let num_required_failing_cases = num_failing_cases - num_soft_failing_cases;
        let num_passing_cases = num_cases - num_failing_cases - num_skipped_cases;
        if num_failing_cases > 0 {
            println!("Section {}", section_output.title);
            for (i, case) in failing_cases {
                let mut soft_flags = vec![];
                if case.soft {
                    soft_flags.push("soft");
                }
                if case.deferrable {
                    soft_flags.push("deferrable");
                }
                if case.optional {
                    soft_flags.push("optional");
                }
                let soft_flags = if soft_flags.is_empty() {
                    "".to_owned()
                } else {
                    format!(" ({})", soft_flags.join(", "))
                };
                println!(
                    "Case {} (line {}){}:",
                    num_cases_seen + i + 1,
                    case.line_number,
                    soft_flags
                );
                println!("Input> {}", case.input.join("\n"));
                println!("Expected output {}", case.expected_output);
                println!("Actual output ;=> {}", case.actual_output);
            }
            println!();
        }
        passing_cases += num_passing_cases;
        soft_fails += num_soft_failing_cases;
        required_fails += num_required_failing_cases;
        num_cases_seen += num_cases;
        skipped_cases += num_skipped_cases;
    }
    println!(
        "Summary: {} cases, {} passed, {} skipped, {} soft fails, {} hard fails, {} total fails",
        num_cases_seen,
        passing_cases,
        skipped_cases,
        soft_fails,
        required_fails,
        required_fails + soft_fails,
    );
}
