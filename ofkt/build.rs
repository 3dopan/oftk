fn main() {
    // Windows環境の場合のみリソースを埋め込む
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        // アイコン設定（将来的に追加予定）
        // res.set_icon("resources/icon.ico");

        // メタデータ設定
        res.set("ProductName", "Ofkt");
        res.set("FileDescription", "軽量ファイル管理ツール");
        res.set("LegalCopyright", "Copyright (C) 2025");
        res.set("CompanyName", "");

        // コンパイル
        res.compile().unwrap();
    }
}
