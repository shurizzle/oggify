extern crate toml;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use directories::ProjectDirs;

lazy_static! {
    pub static ref BASE_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
}

fn proj_dirs() -> ProjectDirs {
    match *BASE_PATH.read().expect("can't readlock BASE_PATH") {
        Some(ref basepath) => ProjectDirs::from_path(basepath.clone()).expect("invalid basepath"),
        None => {
            ProjectDirs::from("org", "affekt", "ncspot").expect("can't determine project paths")
        }
    }
}

pub fn config_path(file: &str) -> PathBuf {
    let proj_dirs = proj_dirs();
    let cfg_dir = proj_dirs.config_dir();
    trace!("{:?}", cfg_dir);
    if cfg_dir.exists() && !cfg_dir.is_dir() {
        fs::remove_file(cfg_dir).expect("unable to remove old config file");
    }
    if !cfg_dir.exists() {
        fs::create_dir(cfg_dir).expect("can't create config folder");
    }
    let mut cfg = cfg_dir.to_path_buf();
    cfg.push(file);
    cfg
}

pub fn load_or_generate_default<
    P: AsRef<Path>,
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: Fn(&Path) -> Result<T, String>,
>(
    path: P,
    default: F,
    default_on_parse_failure: bool,
) -> Result<T, String> {
    let path = path.as_ref();
    // Nothing exists so just write the default and return it
    if !path.exists() {
        let value = default(&path)?;
        return write_content_helper(&path, value);
    }

    // load the serialized content. Always report this failure
    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Unable to read {}: {}", path.to_string_lossy(), e))?;

    // Deserialize the content, optionally fall back to default if it fails
    let result = toml::from_str(&contents);
    if default_on_parse_failure && result.is_err() {
        let value = default(&path)?;
        return write_content_helper(&path, value);
    }
    result.map_err(|e| format!("Unable to parse {}: {}", path.to_string_lossy(), e))
}

fn write_content_helper<P: AsRef<Path>, T: serde::Serialize>(
    path: P,
    value: T,
) -> Result<T, String> {
    let content =
        toml::to_string_pretty(&value).map_err(|e| format!("Failed serializing value: {}", e))?;
    fs::write(path.as_ref(), content)
        .map(|_| value)
        .map_err(|e| {
            format!(
                "Failed writing content to {}: {}",
                path.as_ref().display(),
                e
            )
        })
}
