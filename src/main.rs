use serde::Deserialize;
use std::fs;
use std::process::Command;

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

impl Task {
    fn execute(&self) -> Result<(), std::io::Error> {
        let mut child = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &self.command])
                .spawn()?
        } else {
            Command::new("sh")
                .args(["-c", &self.command])
                .spawn()?
        };

        let status = child.wait()?;
        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Command failed with exit code: {:?}", status.code()),
            ));
        }
        Ok(())
    }
}

fn parse_config(content: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(content)
}

fn main() {
    let config_content = fs::read_to_string("drom.toml").expect("Failed to read drom.toml");
    let config = parse_config(&config_content).expect("Failed to parse drom.toml");
    
    for task in config.tasks {
        println!("Running task: {}", task.name);
        if let Err(e) = task.execute() {
            eprintln!("Error executing task {}: {}", task.name, e);
            std::process::exit(1);
        }
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

    #[test]
    fn test_task_execute_success() {
        let task = Task {
            name: "test".to_string(),
            command: "echo 'success'".to_string(),
        };
        assert!(task.execute().is_ok());
    }

    #[test]
    fn test_task_execute_failure() {
        let task = Task {
            name: "fail".to_string(),
            command: "exit 1".to_string(),
        };
        assert!(task.execute().is_err());
    }
}
