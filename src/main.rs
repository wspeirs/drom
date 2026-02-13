use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(rename = "task")]
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Task {
    name: String,
    command: String,
}

fn parse_config(content: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(content)
}

fn main() {
    let config_content = fs::read_to_string("drom.toml").expect("Failed to read drom.toml");
    let config = parse_config(&config_content).expect("Failed to parse drom.toml");
    
    for task in config.tasks {
        println!("Running task: {} ({})", task.name, task.command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let content = r#"
[[task]]
name = "test"
command = "echo test"
"#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.tasks.len(), 1);
        assert_eq!(config.tasks[0].name, "test");
        assert_eq!(config.tasks[0].command, "echo test");
    }

    #[test]
    fn test_parse_invalid_config() {
        let content = "invalid toml";
        let result = parse_config(content);
        assert!(result.is_err());
    }
}
