use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::thread;

#[derive(Debug, Deserialize)]
struct Config {
    clean: Option<Clean>,
    #[serde(rename = "generate")]
    generate: Option<Vec<Generate>>,
    #[serde(rename = "project")]
    projects: Option<Vec<Project>>,
    #[serde(rename = "group")]
    groups: Option<Vec<Group>>,
}

impl Config {
    fn perform_clean(&self) -> Result<(), std::io::Error> {
        if let Some(clean) = &self.clean {
            let mut handles = vec![];
            for dir in &clean.directories {
                let dir = dir.clone();
                let handle = thread::spawn(move || {
                    if std::path::Path::new(&dir).exists() {
                        println!("Cleaning directory: {}", dir);
                        fs::remove_dir_all(&dir)
                    } else {
                        Ok(())
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "Thread panicked during cleanup")
                })??;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Clean {
    directories: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Generate {
    name: String,
    command: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Project {
    name: String,
    command: String,
    args: Option<Vec<String>>,
    depends_on: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Group {
    name: String,
    projects: Vec<String>,
}

impl Project {
    fn resolve_command(&self, commands: &HashMap<String, String>) -> String {
        let base_command = commands.get(&self.command).cloned().unwrap_or_else(|| self.command.clone());
        if let Some(args) = &self.args {
            format!("{} {}", base_command, args.join(" "))
        } else {
            base_command
        }
    }

    fn execute(&self, commands: &HashMap<String, String>) -> Result<(), std::io::Error> {
        let full_command = self.resolve_command(commands);
        let mut child = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &full_command])
                .spawn()?
        } else {
            Command::new("sh")
                .args(["-c", &full_command])
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

fn parse_commands(content: &str) -> Result<HashMap<String, String>, toml::de::Error> {
    toml::from_str(content)
}

fn main() {
    let config_content = fs::read_to_string("drom.toml").expect("Failed to read drom.toml");
    let config = parse_config(&config_content).expect("Failed to parse drom.toml");
    
    let commands_content = fs::read_to_string("commands.toml").unwrap_or_default();
    let commands = parse_commands(&commands_content).unwrap_or_default();

    if let Err(e) = config.perform_clean() {
        eprintln!("Error during cleanup: {}", e);
        std::process::exit(1);
    }

    if let Some(projects) = config.projects {
        for project in projects {
            println!("Running project: {}", project.name);
            if let Err(e) = project.execute(&commands) {
                eprintln!("Error executing project {}: {}", project.name, e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let content = r#"
[[project]]
name = "test"
command = "echo test"
"#;
        let config = parse_config(content).unwrap();
        let projects = config.projects.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "test");
        assert_eq!(projects[0].command, "echo test");
    }

    #[test]
    fn test_parse_invalid_config() {
        let content = "invalid toml";
        let result = parse_config(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_task_execute_success() {
        let project = Project {
            name: "test".to_string(),
            command: "echo 'success'".to_string(),
            args: None,
            depends_on: None,
        };
        let commands = HashMap::new();
        assert!(project.execute(&commands).is_ok());
    }

    #[test]
    fn test_task_execute_failure() {
        let project = Project {
            name: "fail".to_string(),
            command: "exit 1".to_string(),
            args: None,
            depends_on: None,
        };
        let commands = HashMap::new();
        assert!(project.execute(&commands).is_err());
    }

    #[test]
    fn test_parse_advanced_config() {
        let content = r#"
[clean]
directories = ["dist", "build"]

[[generate]]
name = "proto"
command = "protoc --rust_out=. src/proto/*.proto"

[[project]]
name = "api"
command = "cargo run"
depends_on = ["proto"]

[[group]]
name = "backend"
projects = ["api"]
"#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.clean.as_ref().unwrap().directories.len(), 2);
        assert_eq!(config.generate.as_ref().unwrap().len(), 1);
        assert_eq!(config.projects.as_ref().unwrap().len(), 1);
        assert_eq!(config.groups.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_commands() {
        let content = r#"
mvn = "mvn clean compile"
python = "uv run python"
"#;
        let commands = parse_commands(content).unwrap();
        assert_eq!(commands.get("mvn").unwrap(), "mvn clean compile");
        assert_eq!(commands.get("python").unwrap(), "uv run python");
    }

    #[test]
    fn test_parse_empty_config() {
        let content = "";
        let config = parse_config(content).unwrap();
        assert!(config.clean.is_none());
        assert!(config.generate.is_none());
        assert!(config.projects.is_none());
        assert!(config.groups.is_none());
    }

    #[test]
    fn test_parse_partial_config() {
        let content = r#"
[clean]
directories = ["temp"]
"#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.clean.unwrap().directories, vec!["temp"]);
        assert!(config.generate.is_none());
    }

    #[test]
    fn test_clean_directories() {
        let dirs = vec!["test_dir1".to_string(), "test_dir2".to_string()];
        for dir in &dirs {
            fs::create_dir_all(dir).unwrap();
        }
        
        let config = Config {
            clean: Some(Clean { directories: dirs.clone() }),
            generate: None,
            projects: None,
            groups: None,
        };
        
        config.perform_clean().unwrap();
        
        for dir in &dirs {
            assert!(!std::path::Path::new(dir).exists());
        }
    }

    #[test]
    fn test_resolve_command() {
        let mut commands = HashMap::new();
        commands.insert("mvn".to_string(), "mvn clean compile".to_string());
        
        let project = Project {
            name: "api".to_string(),
            command: "mvn".to_string(),
            args: Some(vec!["test".to_string()]),
            depends_on: None,
        };
        
        let resolved = project.resolve_command(&commands);
        assert_eq!(resolved, "mvn clean compile test");
    }

    #[test]
    fn test_resolve_command_no_alias() {
        let commands = HashMap::new();
        let project = Project {
            name: "api".to_string(),
            command: "ls".to_string(),
            args: Some(vec!["-la".to_string()]),
            depends_on: None,
        };
        let resolved = project.resolve_command(&commands);
        assert_eq!(resolved, "ls -la");
    }

    #[test]
    fn test_clean_missing_directory() {
        let config = Config {
            clean: Some(Clean { directories: vec!["non_existent_dir".to_string()] }),
            generate: None,
            projects: None,
            groups: None,
        };
        assert!(config.perform_clean().is_ok());
    }
}
