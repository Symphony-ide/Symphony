use std::env::current_dir;

use sveditor_core_api::{Manifest, ManifestErrors};

#[tokio::test]
async fn load_manifests() {
    let cwd = current_dir().unwrap();

    let ok_manifest_path = cwd.join("tests/ok_manifest.toml");
    let bad_manifest_path = cwd.join("tests/bad_manifest.toml");
    let not_found_manifest_path = cwd.join("this_doesnt_exist");

    let ok_manifest = Manifest::parse(&ok_manifest_path).await;
    let bad_manifest = Manifest::parse(&bad_manifest_path).await;
    let not_found_manifest = Manifest::parse(&not_found_manifest_path).await;

    assert!(ok_manifest.is_ok());
    assert_eq!(bad_manifest.unwrap_err(), ManifestErrors::CannotParse);
    assert_eq!(not_found_manifest.unwrap_err(), ManifestErrors::NotFound);
}
