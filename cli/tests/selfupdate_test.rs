use open_mtr_cli::selfupdate::{asset_name, extract_tar_gz, parse_checksums, target};

#[test]
fn target_is_supported() {
    let t = target().expect("target");
    assert!(t.contains('-'));
}

#[test]
fn asset_names_follow_platform() {
    assert_eq!(
        asset_name("v0.2.0", "x86_64-unknown-linux-musl"),
        "openmtr-v0.2.0-x86_64-unknown-linux-musl.tar.gz"
    );
    assert_eq!(
        asset_name("v0.2.0", "x86_64-pc-windows-msvc"),
        "openmtr-v0.2.0-x86_64-pc-windows-msvc.zip"
    );
}

#[test]
fn parse_checksums_matches_asset() {
    let text = "abc  other.tar.gz\ndeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef  openmtr-v0.1.0-x86_64-unknown-linux-musl.tar.gz\n";
    let hash =
        parse_checksums(text, "openmtr-v0.1.0-x86_64-unknown-linux-musl.tar.gz").expect("hash");
    assert_eq!(
        hash,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    );
}

#[test]
fn parse_checksums_rejects_missing() {
    assert!(parse_checksums("", "x.tar.gz").is_none());
}

#[test]
fn extract_reads_first_openmtr_entry() {
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    let data = b"fake-binary";
    header.set_size(data.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "openmtr", &data[..])
        .expect("append");
    let uncompressed = tar.into_inner().expect("inner");
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut gz, &uncompressed).expect("gz");
    let bytes = gz.finish().expect("finish");
    assert_eq!(extract_tar_gz(&bytes).expect("extract"), data);
}

#[cfg(unix)]
#[tokio::test]
async fn installs_verified_tarball_atomically() {
    use open_mtr_cli::selfupdate::{UpdateOptions, asset_name, target};
    use sha2::{Digest, Sha256};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let binary = b"fake-openmtr-binary".to_vec();
    let mut tar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(binary.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    tar.append_data(&mut header, "openmtr", &binary[..])
        .expect("append");
    let uncompressed = tar.into_inner().expect("inner");
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut gz, &uncompressed).expect("gz");
    let tarball = gz.finish().expect("finish");

    let target = target().expect("target");
    let asset = asset_name("v9.9.9", &target);
    let hash = hex::encode(Sha256::digest(&tarball));
    let checksums = format!("{hash}  {asset}\n");

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/st-all-one/open-mtr-rs/releases/download/v9.9.9/sha256sums.txt",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(checksums))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/st-all-one/open-mtr-rs/releases/download/v9.9.9/{asset}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(tarball))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().expect("tempdir");
    let install_path = dir.path().join("bin").join("openmtr");
    std::fs::create_dir_all(install_path.parent().expect("parent")).expect("mkdir");

    let opts = UpdateOptions {
        version: Some("v9.9.9".to_string()),
        download_base: Some(server.uri()),
        install_path: Some(install_path.clone()),
        ..Default::default()
    };
    let plan = open_mtr_cli::selfupdate::run(&opts).await.expect("update");
    assert!(plan.changed);
    assert_eq!(std::fs::read(&install_path).expect("bin"), binary);
}
