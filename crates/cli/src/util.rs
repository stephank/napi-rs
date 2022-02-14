use std::{
  env,
  fs::{self, File},
  io::{self, Write},
  path::Path,
};

pub type CommandResult = Result<(), ()>;

pub trait Executable {
  fn execute(&mut self) -> CommandResult;
}

pub const AVAILABLE_TARGETS: &[&str] = &[
  "aarch64-apple-darwin",
  "aarch64-linux-android",
  "aarch64-unknown-linux-gnu",
  "aarch64-unknown-linux-musl",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-unknown-freebsd",
  "i686-pc-windows-msvc",
  "armv7-unknown-linux-gnueabihf",
  "armv7-linux-androideabi",
];

pub const DEFAULT_TARGETS: &[&str] = &[
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

pub fn write_file<P: AsRef<Path>>(path: &P, content: &str) -> Result<(), io::Error> {
  let path = path.as_ref();
  println!("Writing file: {}", path.display());
  if env::var("NAPI_DEBUG").is_ok() {
    println!("{}", &content);
  } else {
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
  }

  Ok(())
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum NodeArch {
  x32,
  x64,
  ia32,
  arm,
  arm64,
  mips,
  mipsel,
  ppc,
  ppc64,
  s390,
  s390x,
}

impl NodeArch {
  fn from_str(s: &str) -> Option<Self> {
    match s {
      "x32" => Some(NodeArch::x32),
      "x86_64" => Some(NodeArch::x64),
      "i686" => Some(NodeArch::ia32),
      "armv7" => Some(NodeArch::arm),
      "arrch64" => Some(NodeArch::arm64),
      "mips" => Some(NodeArch::mips),
      "mipsel" => Some(NodeArch::mipsel),
      "ppc" => Some(NodeArch::ppc),
      "ppc64" => Some(NodeArch::ppc64),
      "s390" => Some(NodeArch::s390),
      "s390x" => Some(NodeArch::s390x),
      _ => None,
    }
  }
}

impl std::fmt::Display for NodeArch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodeArch::x32 => write!(f, "x32"),
      NodeArch::x64 => write!(f, "x64"),
      NodeArch::ia32 => write!(f, "ia32"),
      NodeArch::arm => write!(f, "arm"),
      NodeArch::arm64 => write!(f, "arm64"),
      NodeArch::mips => write!(f, "mips"),
      NodeArch::mipsel => write!(f, "mipsel"),
      NodeArch::ppc => write!(f, "ppc"),
      NodeArch::ppc64 => write!(f, "ppc64"),
      NodeArch::s390 => write!(f, "s390"),
      NodeArch::s390x => write!(f, "s390x"),
    }
  }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum NodePlatform {
  darwin,
  freebsd,
  openbsd,
  win32,
  unknown(String),
}

impl NodePlatform {
  fn from_str(s: &str) -> Self {
    match s {
      "darwin" => NodePlatform::darwin,
      "freebsd" => NodePlatform::freebsd,
      "openbsd" => NodePlatform::openbsd,
      "windows" => NodePlatform::win32,
      _ => NodePlatform::unknown(s.to_owned()),
    }
  }
}

impl std::fmt::Display for NodePlatform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodePlatform::darwin => write!(f, "darwin"),
      NodePlatform::freebsd => write!(f, "freebsd"),
      NodePlatform::openbsd => write!(f, "openbsd"),
      NodePlatform::win32 => write!(f, "win32"),
      NodePlatform::unknown(s) => write!(f, "{}", s),
    }
  }
}

pub struct PlatformDetail {
  pub triple: String,
  pub platform_abi: String,
  pub arch: NodeArch,
  pub platform: NodePlatform,
  pub abi: Option<String>,
}

impl From<&str> for PlatformDetail {
  fn from(triple: &str) -> PlatformDetail {
    let parts = triple.split('-').collect::<Vec<_>>();
    let (cpu, sys, abi) = if parts.len() == 2 {
      (parts[0], parts[2], None)
    } else {
      (parts[0], parts[2], parts.get(3))
    };

    let platform = NodePlatform::from_str(sys);
    let arch = NodeArch::from_str(cpu).unwrap_or_else(|| panic!("unsupported cpu arch {}", cpu));
    PlatformDetail {
      triple: triple.to_string(),
      platform_abi: if abi.is_some() {
        format!("{}-{}-{}", platform, arch, abi.unwrap())
      } else {
        format!("{}-{}", platform, arch)
      },
      arch,
      platform,
      abi: abi.map(|s| s.to_string()),
    }
  }
}
