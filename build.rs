use fs_extra::dir::CopyOptions;
use glob;
use std::{
    env,
    error::Error,
    fmt,
    fs::File,
    io::Write,
    path::{Path, StripPrefixError},
    process::Command,
};

#[derive(Debug)]
struct BuildError(String);

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BuildError: {}", self.0)
    }
}
impl Error for BuildError {}
impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError(format!("stri-prefix-error: {}", err))
    }
}
impl From<StripPrefixError> for BuildError {
    fn from(err: StripPrefixError) -> Self {
        BuildError(format!("io-error: {}", err))
    }
}

const UI_PACKAGE_JSON: &str = "ui/package.json";
const UI_TSCONFIG: &str = "ui/tsconfig.json";
const UI_YARN_LOCK: &str = "ui/yarn.lock";
const UI_SRC_DIR: &str = "ui/src";
const UI_PUBLIC_DIR: &str = "ui/public";
const UI_TARGET_PATTERNS: [&str; 7] = [
    "static/css/**/*.css",
    "static/js/**/*.js",
    "*.ico",
    "*.ico",
    "*.png",
    "*.json",
    "*.html",
];

fn append_file_bytes(
    file: &mut File,
    out_path: &Path,
    glob_pattern: &str,
) -> Result<(), BuildError> {
    let build_path = out_path.join("ui/build");
    let glob_pattern = build_path.join(glob_pattern);
    let pattern = glob_pattern.to_str().to_owned().unwrap();
    let paths = glob::glob(pattern).unwrap();

    for result in paths {
        match result {
            // Trigger a full project rebuild if any of the files printed below has changed
            Ok(path) => {
                writeln!(
                    file,
                    r##"("{name}", include_bytes!(r#"{file_path}"#)),"##,
                    name = path.strip_prefix(&build_path)?.to_str().unwrap(),
                    file_path = path.to_str().unwrap(),
                )?;
            }
            Err(err) => return Err(BuildError(format!("glob error: {:?}", err))),
        }
    }
    Ok(())
}

fn main() -> Result<(), BuildError> {
    for pattern in &[
        UI_PACKAGE_JSON,
        UI_TSCONFIG,
        UI_YARN_LOCK,
        &format!("{}/**/*", UI_PUBLIC_DIR),
        &format!("{}/**/*", UI_SRC_DIR),
    ] {
        let paths = glob::glob(pattern).unwrap();
        for result in paths {
            match result {
                // Trigger a full project rebuild if any of the files printed below has changed
                Ok(path) => println!("cargo:rerun-if-changed={}", path.to_str().unwrap()),
                Err(err) => return Err(BuildError(format!("glob error: {:?}", err))),
            }
        }
    }
    let out_dir = env::var("OUT_DIR").unwrap();
    let ui_dir = Path::new(&out_dir).join("ui");

    std::fs::create_dir_all(ui_dir.clone())?;

    let copy_options = CopyOptions {
        overwrite: false,
        skip_exist: true,
        buffer_size: 64000,
        copy_inside: true,
        depth: 0,
    };
    fs_extra::copy_items(
        &vec![UI_PACKAGE_JSON, UI_TSCONFIG, UI_PUBLIC_DIR, UI_SRC_DIR],
        ui_dir.clone(),
        &copy_options,
    )
    .unwrap();

    Command::new("npm")
        .args(&["install"])
        .current_dir(ui_dir.clone())
        .spawn()
        .expect("failed to run npm install");

    let exit_status = Command::new("npx")
        .args(&["react-scripts", "build"])
        .current_dir(ui_dir.clone())
        .status()?;

    match exit_status.code() {
        Some(0) => {
            let dest_path = Path::new(&out_dir).join("ui_build_assets.rs");
            let out_dir = Path::new(&out_dir);

            let mut all_the_files = File::create(&dest_path)?;
            writeln!(&mut all_the_files, r##"["##,)?;

            for pattern in UI_TARGET_PATTERNS.to_vec() {
                append_file_bytes(&mut all_the_files, &out_dir, pattern)?;
            }

            writeln!(&mut all_the_files, r##"]"##,)?;

            Ok(())
        }
        other => Err(BuildError(format!(
            "`yarn build` command exited with an unexpected status code: {:?}",
            other
        ))),
    }
}
