use std::io;
use std::process::{Child, Command};

use cucumber::gherkin::Step;
use cucumber::{given, then, when, World as _};
use reqwest::Client;

#[derive(cucumber::World, Debug, Default)]
pub struct World {
  config_file: String,
  server_started: bool,
  response_text: Option<String>,
  server_child: Option<Child>,
}

impl World {
  pub fn cleanup(&mut self) -> io::Result<()> {
    if let Some(child) = self.server_child.as_mut() {
      child.kill()?;
      child.wait()?;
      self.server_child = None;
    }
    Ok(())
  }
}

impl Drop for World {
  fn drop(&mut self) {
    if let Err(err) = self.cleanup() {
      eprintln!("Error cleaning up server child process: {}", err);
    }
  }
}

#[given(regex = "I have a config file at \"(.*)\"")]
fn check_config_file(world: &mut World, config_path: String) {
  assert!(std::path::Path::new(&config_path).exists());
  world.config_file = config_path;
}

#[when(regex = "I run the application with \"(.*)\"")]
async fn start_server(world: &mut World, command: String) {
  let command_parts: Vec<&str> = command.split_whitespace().collect();
  let mut cmd = Command::new(command_parts[0]);

  for arg in command_parts.iter().skip(1) {
    cmd.arg(arg);
  }

  let child = cmd.spawn().expect("failed to start server");
  world.server_child = Some(child);
  tokio::time::sleep(std::time::Duration::from_secs(2)).await; // Give the server time to start
  world.server_started = true;
}

#[then(regex = "I can make a HTTP \"(.*)\" request on port \"(\\d+)\" for path \"(.*)\" with body")]
async fn make_http_call(world: &mut World, step: &Step, method: String, port: u16, path: String) {
  let body = step.docstring().unwrap().clone();
  assert!(world.server_started);
  let url = format!("http://localhost:{}{}", port, path);
  let client = Client::new();

  let response = match method.to_uppercase().as_str() {
    "POST" => client.post(&url).body(body).send(),
    _ => panic!("unsupported HTTP method"),
  };

  let response = response.await.unwrap();

  assert!(response.status().is_success());
  let response_text = response.text().await.unwrap();
  world.response_text = Some(response_text);
}

#[then(regex = "the response should contain")]
async fn check_response(world: &mut World, step: &Step) {
  let expected = step.docstring().unwrap();
  let expected = expected.replace('\n', "");
  let response_text = world.response_text.as_ref().unwrap();
  assert!(response_text.contains(&expected));
}

#[tokio::main]
async fn main() {
  World::cucumber().run("features").await;
}
