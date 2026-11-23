use super::release_tag;

#[test]
fn release_tag_accepts_with_or_without_v() {
    assert_eq!(release_tag("0.2.0"), "v0.2.0");
    assert_eq!(release_tag("v0.2.0"), "v0.2.0");
    assert_eq!(release_tag(" v0.2.0 "), "v0.2.0");
}
