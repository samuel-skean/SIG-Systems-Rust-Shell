// Disclaimer: This test suite was written by an LLM
#[cfg(test)]
mod tests {
    use crate::parser::{CommandGroup, RedirType};

    use std::path::PathBuf;

    fn assert_single_command(group: &CommandGroup, expected_args: &[&str]) {
        assert_eq!(group.commands.len(), 1, "Expected exactly one command");
        assert_eq!(
            group.commands[0].argv,
            expected_args.iter().map(|&s| s.to_string()).collect::<Vec<_>>()
        );
    }

    fn assert_pipe_chain(group: &CommandGroup, commands: &[&[&str]]) {
        let mut current = &group.commands[0];
        for (i, expected_args) in commands.iter().enumerate() {
            assert_eq!(
                current.argv,
                expected_args.iter().map(|&s| s.to_string()).collect::<Vec<_>>(),
                "Mismatch in command {} of pipe chain", i
            );
            
            if i < commands.len() - 1 {
                let pipe_to = current.pipe_to.as_ref().expect("Expected pipe to next command");
                current = &pipe_to.target;
            } else {
                assert!(current.pipe_to.is_none(), "Unexpected pipe at end of chain");
            }
        }
    }

    #[test]
    fn test_basic_command() {
        let group = CommandGroup::parse("ls -la");
        assert_single_command(&group, &["ls", "-la"]);
    }

    #[test]
    fn test_command_with_numbers() {
        let group = CommandGroup::parse("echo 123 456");
        assert_single_command(&group, &["echo", "123", "456"]);
    }

    #[test]
    fn test_quoted_arguments() {
        let tests = vec![
            ("echo 'hello world'", vec!["echo", "hello world"]),
            ("echo \"hello world\"", vec!["echo", "hello world"]),
            ("echo 'hello\"world'", vec!["echo", "hello\"world"]),
            ("echo \"hello'world\"", vec!["echo", "hello'world"]),
            ("echo 'hello\\world'", vec!["echo", "hello\\world"]),
        ];

        for (input, expected) in tests {
            let group = CommandGroup::parse(input);
            assert_single_command(&group, &expected);
        }
    }

    #[test]
    fn test_mixed_quotes() {
        let group = CommandGroup::parse("echo 'single' \"double\" normal 'mixed\"quotes'");
        assert_single_command(&group, &["echo", "single", "double", "normal", "mixed\"quotes"]);
    }

    #[test]
    fn test_simple_pipe() {
        let group = CommandGroup::parse("ls -l | grep foo");
        assert_pipe_chain(&group, &[&["ls", "-l"], &["grep", "foo"]]);
    }

    #[test]
    fn test_multiple_pipes() {
        let group = CommandGroup::parse("cat file.txt | grep error | wc -l");
        assert_pipe_chain(&group, &[&["cat", "file.txt"], &["grep", "error"], &["wc", "-l"]]);
    }

    #[test]
    fn test_pipe_both() {
        let group = CommandGroup::parse("cmd1 |& cmd2");
        let pipe_to = group.commands[0].pipe_to.as_ref().unwrap();
        assert!(matches!(pipe_to.pipe_type, RedirType::Both));
    }

    #[test]
    fn test_stdout_redirection() {
        let tests = vec![
            "echo hello > output.txt",
            "echo hello 1> output.txt",
            "echo hello >> output.txt",
            "echo hello 1>> output.txt",
        ];

        for input in tests {
            let group = CommandGroup::parse(input);
            assert_eq!(group.commands[0].argv, vec!["echo", "hello"]);
            assert_eq!(group.commands[0].redirect_to.len(), 1);
            assert!(matches!(
                group.commands[0].redirect_to[0].redirect_type,
                RedirType::Stdout
            ));
            assert_eq!(
                group.commands[0].redirect_to[0].target,
                PathBuf::from("output.txt")
            );
        }
    }

    #[test]
    fn test_stderr_redirection() {
        let tests = vec![
            "cmd 2> error.log",
            "cmd 2>> error.log",
        ];

        for input in tests {
            let group = CommandGroup::parse(input);
            assert_eq!(group.commands[0].argv, vec!["cmd"]);
            assert_eq!(group.commands[0].redirect_to.len(), 1);
            assert!(matches!(
                group.commands[0].redirect_to[0].redirect_type,
                RedirType::Stderr
            ));
            assert_eq!(
                group.commands[0].redirect_to[0].target,
                PathBuf::from("error.log")
            );
        }
    }

    #[test]
    fn test_both_redirection() {
        let tests = vec![
            "cmd &> both.log",
            "cmd &>> both.log",
        ];

        for input in tests {
            let group = CommandGroup::parse(input);
            assert_eq!(group.commands[0].argv, vec!["cmd"]);
            assert_eq!(group.commands[0].redirect_to.len(), 1);
            assert!(matches!(
                group.commands[0].redirect_to[0].redirect_type,
                RedirType::Both
            ));
            assert_eq!(
                group.commands[0].redirect_to[0].target,
                PathBuf::from("both.log")
            );
        }
    }

    #[test]
    fn test_multiple_redirections() {
        let group = CommandGroup::parse("cmd > stdout.log 2> stderr.log");
        assert_eq!(group.commands[0].argv, vec!["cmd"]);
        assert_eq!(group.commands[0].redirect_to.len(), 2);
        
        assert!(matches!(
            group.commands[0].redirect_to[0].redirect_type,
            RedirType::Stdout
        ));
        assert_eq!(
            group.commands[0].redirect_to[0].target,
            PathBuf::from("stdout.log")
        );
        
        assert!(matches!(
            group.commands[0].redirect_to[1].redirect_type,
            RedirType::Stderr
        ));
        assert_eq!(
            group.commands[0].redirect_to[1].target,
            PathBuf::from("stderr.log")
        );
    }

    #[test]
    fn test_complex_commands() {
        let complex_tests = vec![
            (
                "find . -name '*.rs' | xargs grep 'fn main' > output.txt 2> error.log",
                vec![
                    vec!["find", ".", "-name", "*.rs"],
                    vec!["xargs", "grep", "fn main"],
                ],
            ),
            (
                "cat file.txt | grep -v error |& wc -l > count.txt",
                vec![
                    vec!["cat", "file.txt"],
                    vec!["grep", "-v", "error"],
                    vec!["wc", "-l"],
                ],
            ),
            (
                "echo 'Hello, world!' > greeting.txt | cat greeting.txt | tr '[:lower:]' '[:upper:]'",
                vec![
                    vec!["echo", "Hello, world!"],
                    vec!["cat", "greeting.txt"],
                    vec!["tr", "[:lower:]", "[:upper:]"],
                ],
            ),
        ];

        for (input, expected_commands) in complex_tests {
            let group = CommandGroup::parse(input);
            assert_pipe_chain(&group, &expected_commands.iter().map(|v| v.as_slice()).collect::<Vec<_>>());
        }
    }

    #[test]
    fn test_empty_input() {
        let group = CommandGroup::parse("");
        assert!(group.commands.is_empty());
        
        let group = CommandGroup::parse("   ");
        assert!(group.commands.is_empty());
    }

    #[test]
    fn test_invalid_redirections() {
        let tests = vec![
            "cmd >",           // Missing filename
            "cmd 2>",          // Missing filename
            "cmd &>",          // Missing filename
            "cmd > > file",    // Double redirection
            "cmd 2> > file",   // Invalid redirection syntax
        ];

        for input in tests {
            let group = CommandGroup::parse(input);
            assert!(group.commands.is_empty(), "Expected empty command group for invalid input: {}", input);
        }
    }

    #[test]
    fn test_whitespace_handling() {
        let tests = vec![
            "cmd   arg1    arg2",
            "\tcmd\targ1\targ2",
            "cmd \t arg1 \t arg2",
            "  cmd  arg1  arg2  ",
        ];

        for input in tests {
            let group = CommandGroup::parse(input);
            assert_single_command(&group, &["cmd", "arg1", "arg2"]);
        }
    }
}
