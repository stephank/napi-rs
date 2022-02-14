use clap::Args;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use minijinja::{context, Environment};
use std::{fs, io};

use crate::util::{write_file, CommandResult, Executable, AVAILABLE_TARGETS, DEFAULT_TARGETS};

#[derive(Args, Debug)]
#[clap(version)]
/// create a new project with pre-configured boilerplate
pub struct NewCommand {
  /// the path where the napi-rs crate will be created
  path: String,

  #[clap(short, long)]
  /// name of the napi-rs crate
  name: Option<String>,

  #[clap(short, long)]
  /// all targets the crate will be compiled for. Use `--default-targets` to use the default ones.
  targets: Option<Vec<String>>,

  #[clap(long)]
  /// whether enable default targets
  enable_default_targets: bool,

  #[clap(long)]
  /// whether enable all targets
  enable_all_targets: bool,

  #[clap(short = 'd', long)]
  /// whether enable the `type-def` feature for typescript definitions auto-generation
  type_def: bool,

  #[clap(long)]
  /// whether generate preconfigured github actions to crate folder
  enable_github_actions: bool,
}

impl Executable for NewCommand {
  fn execute(&mut self) -> CommandResult {
    if let Err(e) = fs::create_dir_all(&self.path) {
      eprintln!("{}", e);
      eprintln!("Failed to create directory {}", self.path);

      return Err(());
    }

    self.fetch_name();
    self.fetch_targets();

    if let Err(e) = self.write_files() {
      eprintln!("{}", e);
      return Err(());
    }
    Ok(())
  }
}

impl NewCommand {
  fn fetch_name(&mut self) {
    self.name.get_or_insert_with(|| {
      Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Package name (The name filed in your package.json)")
        .default(self.path.clone())
        .interact_text()
        .unwrap()
    });
  }

  fn fetch_targets(&mut self) {
    self.targets.get_or_insert_with(|| {
      let mut targets: Vec<String> = Vec::new();
      if self.enable_default_targets {
        DEFAULT_TARGETS
          .iter()
          .for_each(|t| targets.push(t.to_string()));
      } else if self.enable_all_targets {
        for target in AVAILABLE_TARGETS.iter() {
          targets.push(target.to_string());
        }
      } else {
        loop {
          let selected_target_indices = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(
              "Choose target(s) you want to support ([space] to select, [enter] to confirm)",
            )
            .clear(true)
            .items(AVAILABLE_TARGETS)
            .defaults(
              &AVAILABLE_TARGETS
                .iter()
                .map(|t| DEFAULT_TARGETS.contains(t))
                .collect::<Vec<bool>>(),
            )
            .report(true)
            .interact()
            .unwrap();

          if !selected_target_indices.is_empty() {
            for index in selected_target_indices {
              targets.push(AVAILABLE_TARGETS[index].to_string());
            }
            break;
          }
        }
      }

      targets
    });
  }

  fn write_files(&self) -> io::Result<()> {
    let name = self.name.as_ref().unwrap();
    let targets = self.targets.as_ref().unwrap();
    let mut env = Environment::new();

    self.write_cargo_toml(&mut env, name)?;
    self.write_lib_files(&mut env)?;
    self.write_package_json(&mut env, name, targets)?;

    Ok(())
  }

  fn write_cargo_toml(&self, env: &mut Environment, name: &str) -> io::Result<()> {
    let file_name = "Cargo.toml";
    env
      .add_template(file_name, include_str!("new/templates/cargo_toml"))
      .unwrap();
    let template = env.get_template(file_name).unwrap();
    let cargo_toml = template
      .render(context!(
        name => package_name_to_crate_name(name),
        license => "MIT",
        napi_version => 2,
        napi_derive_version => 2,
        napi_build_version => 1
      ))
      .unwrap();

    write_file(&format!("{}/{file_name}", self.path), &cargo_toml)
  }

  fn write_lib_files(&self, _env: &mut Environment) -> io::Result<()> {
    write_file(
      &format!("{}/src/lib.rs", self.path),
      include_str!("new/templates/lib_rs"),
    )?;

    write_file(
      &format!("{}/build.rs", self.path),
      include_str!("new/templates/build_rs"),
    )?;

    Ok(())
  }

  fn write_package_json(
    &self,
    env: &mut Environment,
    name: &str,
    targets: &[String],
  ) -> io::Result<()> {
    let file_name = "package.json";
    env
      .add_template(file_name, include_str!("new/templates/package_json"))
      .unwrap();

    let template = env.get_template(file_name).unwrap();
    let package_json = template
      .render(context!(
        name => name,
        binary_name => package_name_to_binary_name(name),
        targets => targets,
        license => "MIT",
        min_node_version => "10",
      ))
      .unwrap();

    write_file(&format!("{}/{file_name}", self.path), &package_json)
  }
}

fn package_name_to_crate_name(name: &str) -> String {
  name.trim_start_matches('@').replace("/", "-")
}

fn package_name_to_binary_name(name: &str) -> String {
  name.split('/').last().unwrap().to_string()
}
