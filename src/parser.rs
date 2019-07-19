use toml::Value;
use toml::de::Error;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct TestModule {
    version: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    setup: Option<Vec<Command>>,
    
    #[serde(alias = "test")]
    tests: Vec<Test>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    teardown: Option<Vec<Command>>,
}

#[derive(Deserialize, Debug)]
struct Test {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(alias = "command")]
    commands: Vec<Command>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct Command {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    
    #[serde(alias = "cmd")]
    command: String,
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
  description: String,
  line_col: Option<(usize, usize)>,
}

//TOML Parser
fn parse_toml(config: String) -> Result<TestModule, ParseError> {
    toml::from_str(&config)
      .map_err(|e| {
        println!("HELLO {}", e);

        ParseError {
          description: e.to_string(),
          line_col: e.line_col(),
        }
      })
}

#[test]
fn test_basics() {
    // {
    //   "version": "2",
    //   "setup": [
    //     { "cmd": "abc" },
    //     { "cmd": "abc" }
    //   ],
    //   "tests": [
    //     { "name": "test1",
    //       "commands": [
    //         { "name": "curl", "cmd": "curl google.com" }
    //       ]
    //     }
    //   ]
    // }

    let config = parse_toml(r#"
        version = "3"

        [[setup]]
        name = "setup 1"
        cmd = "abc"
        [[setup]]
        command = "def"

        [[test]]
        name = "test 1"
        [[test.command]]
        name = "curl"
        cmd = "curl google.com"
        [[test.command]]
        name = "ping"
        cmd = """
ping google.com;
curl google.com"""

        [[teardown]]
        name = "teardown 1"
        cmd = "abc"
        [[teardown]]
        command = "def"

    "#.to_string()).unwrap();

    assert_eq!(config.version, "3");
    
    config.setup.map(|a| {
        assert_eq!(a[0].name, Some("setup 1".to_string()));
        assert_eq!(a[0].command, "abc");
        assert_eq!(a[1].command, "def");
    });

    assert_eq!(config.tests[0].name, Some("test 1".to_string()));
    assert_eq!(config.tests[0].commands[0].name, Some("curl".to_string()));
    assert_eq!(config.tests[0].commands[0].command, "curl google.com");
    assert_eq!(config.tests[0].commands[1].name, Some("ping".to_string()));
    assert_eq!(config.tests[0].commands[1].command, "ping google.com;\ncurl google.com");

    config.teardown.map(|a| {
        assert_eq!(a[0].name, Some("teardown 1".to_string()));
        assert_eq!(a[0].command, "abc");
        assert_eq!(a[1].command, "def");
    });
}

#[test]
fn t_setup_and_teardown_optional() {
    let config = parse_toml(r#"
        version = "3"

        [[test]]
        name = "test1"
        [[test.commands]]
        name = "curl"
        cmd = "curl google.com"

    "#.to_string()).unwrap();

    assert_eq!(config.setup, Option::None);
    assert_eq!(config.teardown, Option::None);
}


#[test]
fn t_parse_error() {
    let err: ParseError = parse_toml(r#"
        version = "3"

        garbage

    "#.to_string()).expect_err("Should have failed");

    assert_eq!(err, ParseError {
          description: "expected an equals, found a newline at line 4".to_string(),
          line_col: Some((3, 15)),
        });
}
