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

fn run_command(command: &str) -> Result<(), std::io::Error> {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .spawn()?
    } else {
        Command::new("sh")
            .args(["-c", command])
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

    fn execute_all(&self, commands: &HashMap<String, String>) -> Result<(), std::io::Error> {
        self.perform_clean()?;

        let mut completed_generate = std::collections::HashSet::new();

        if let Some(generate_tasks) = &self.generate {
            for task in generate_tasks {
                task.execute()?;
                completed_generate.insert(task.name.clone());
            }
        }

        if let Some(projects) = &self.projects {
            for project in projects {
                if let Some(deps) = &project.depends_on {
                    for dep in deps {
                        if !completed_generate.contains(dep) {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Dependency '{}' for project '{}' not found or failed", dep, project.name),
                            ));
                        }
                    }
                }
                project.execute(commands)?;
            }
        }

        Ok(())
    }

    fn get_group_projects(&self, group_name: &str) -> Option<Vec<&Project>> {
        let group = self.groups.as_ref()?.iter().find(|g| g.name == group_name)?;
        let mut group_projects = vec![];
        if let Some(projects) = &self.projects {
            for project_name in &group.projects {
                if let Some(project) = projects.iter().find(|p| p.name == *project_name) {
                    group_projects.push(project);
                }
            }
        }
        Some(group_projects)
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

impl Generate {
    fn execute(&self) -> Result<(), std::io::Error> {
        println!("Running generate task: {}", self.name);
        run_command(&self.command)
    }
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
        println!("Running project: {}", self.name);
        let full_command = self.resolve_command(commands);
        run_command(&full_command)
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

    if let Err(e) = config.execute_all(&commands) {
        eprintln!("Execution error: {}", e);
        std::process::exit(1);
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

    #[test]
    fn test_dependency_execution() {
        let content = r#"
[[generate]]
name = "gen"
command = "echo 'generating' > gen.txt"

[[project]]
name = "proj"
command = "cat"
args = ["gen.txt"]
depends_on = ["gen"]
"#;
        let config = parse_config(content).unwrap();
        let commands = HashMap::new();
        
        // This will fail until dependencies are implemented
        config.execute_all(&commands).unwrap();
        
        assert!(std::path::Path::new("gen.txt").exists());
        fs::remove_file("gen.txt").unwrap();
    }

    #[test]
    fn test_get_group_projects() {
        let content = r#"
[[project]]
name = "api"
command = "echo api"

[[project]]
name = "web"
command = "echo web"

[[group]]
name = "backend"
projects = ["api"]
"#;
        let config = parse_config(content).unwrap();
        let backend_projects = config.get_group_projects("backend").unwrap();
        assert_eq!(backend_projects.len(), 1);
        assert_eq!(backend_projects[0].name, "api");
        
        assert!(config.get_group_projects("non_existent").is_none());
    }

    #[test]
    fn test_failing_dependency() {
        let content = r#"
[[generate]]
name = "fail_gen"
command = "exit 1"

[[project]]
name = "proj"
command = "echo success"
depends_on = ["fail_gen"]
"#;
        let config = parse_config(content).unwrap();
        let commands = HashMap::new();
        assert!(config.execute_all(&commands).is_err());
    }
}
