use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread::JoinHandle;

#[cfg(target_os = "windows")]
use std::sync::mpsc::channel;
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

/// 画面端検出機能を提供する構造体
///
/// カーソルが画面右端に一定時間（300ms）留まった場合に検出し、
/// イベントとしてメインスレッドに通知する。
pub struct EdgeDetector {
    sender: Option<Sender<bool>>,
    receiver: Option<Receiver<bool>>,
    thread_handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl EdgeDetector {
    /// 新しいEdgeDetectorインスタンスを作成する
    pub fn new() -> Self {
        Self {
            sender: None,
            receiver: None,
            thread_handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 画面端検出を開始する
    ///
    /// バックグラウンドスレッドを起動し、50ms間隔でカーソル位置をポーリングする。
    /// カーソルが画面右端（画面幅-1以上）に300ms以上留まった場合、イベントを送信する。
    ///
    /// # 戻り値
    ///
    /// * `Ok(())` - 正常に起動した場合
    /// * `Err(String)` - 既に起動している場合
    #[cfg(target_os = "windows")]
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("既に起動しています".to_string());
        }

        let (tx, rx) = channel();
        self.sender = Some(tx.clone());
        self.receiver = Some(rx);

        let running = Arc::clone(&self.running);
        running.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            let mut last_trigger: Option<Instant> = None;
            let mut event_sent = false;

            while running.load(Ordering::Relaxed) {
                // カーソル位置を取得
                let cursor_pos = match get_cursor_pos() {
                    Ok(pos) => pos,
                    Err(_) => {
                        thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                };

                let screen_width = get_screen_width();

                // 右端にいるか判定（画面幅-1以上）
                if cursor_pos.x >= screen_width - 1 {
                    if last_trigger.is_none() {
                        last_trigger = Some(Instant::now());
                        event_sent = false;
                    } else if !event_sent && last_trigger.unwrap().elapsed() >= Duration::from_millis(300) {
                        // 300ms以上右端にいたらトリガー（1回のみ）
                        let _ = tx.send(true);
                        event_sent = true;
                    }
                } else {
                    // 右端から離れたらリセット
                    last_trigger = None;
                    event_sent = false;
                }

                thread::sleep(Duration::from_millis(50));
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// 画面端検出を開始する（非Windows環境用のスタブ）
    #[cfg(not(target_os = "windows"))]
    pub fn start(&mut self) -> Result<(), String> {
        Err("Windows以外のプラットフォームではサポートされていません".to_string())
    }

    /// イベントを処理する
    ///
    /// チャネルから画面端到達イベントを受信する。
    /// ノンブロッキングで、イベントがない場合は即座にfalseを返す。
    ///
    /// # 戻り値
    ///
    /// * `true` - 画面右端に到達した
    /// * `false` - イベントなし
    pub fn handle_events(&self) -> bool {
        if let Some(ref rx) = self.receiver {
            if let Ok(true) = rx.try_recv() {
                return true;
            }
        }
        false
    }

    /// 画面端検出を停止する
    ///
    /// バックグラウンドスレッドを安全に終了させる。
    pub fn stop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);

            // スレッドの終了を待機
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }

            self.sender = None;
            self.receiver = None;
        }
    }
}

impl Default for EdgeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EdgeDetector {
    fn drop(&mut self) {
        self.stop();
    }
}

// Windows専用のヘルパー関数

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::POINT;

/// カーソルの現在位置を取得する
///
/// # 戻り値
///
/// * `Ok(POINT)` - カーソル位置
/// * `Err(String)` - 取得に失敗した場合
#[cfg(target_os = "windows")]
fn get_cursor_pos() -> Result<POINT, String> {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };
        match GetCursorPos(&mut point) {
            Ok(_) => Ok(point),
            Err(_) => Err("カーソル位置の取得に失敗しました".to_string()),
        }
    }
}

/// 画面の幅を取得する
///
/// # 戻り値
///
/// 画面の幅（ピクセル単位）
#[cfg(target_os = "windows")]
fn get_screen_width() -> i32 {
    unsafe {
        GetSystemMetrics(SM_CXSCREEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_edge_detector_new() {
        let detector = EdgeDetector::new();
        assert!(!detector.running.load(Ordering::Relaxed));
        assert!(detector.sender.is_none());
        assert!(detector.receiver.is_none());
        assert!(detector.thread_handle.is_none());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_edge_detector_start() {
        let mut detector = EdgeDetector::new();
        let result = detector.start();
        assert!(result.is_ok());
        assert!(detector.running.load(Ordering::Relaxed));
        assert!(detector.sender.is_some());
        assert!(detector.receiver.is_some());
        assert!(detector.thread_handle.is_some());
        detector.stop();
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_edge_detector_start_non_windows() {
        let mut detector = EdgeDetector::new();
        let result = detector.start();
        assert!(result.is_err());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_edge_detector_start_twice() {
        let mut detector = EdgeDetector::new();
        let _ = detector.start();
        let result = detector.start();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "既に起動しています");
        detector.stop();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_edge_detector_handle_events_no_event() {
        let mut detector = EdgeDetector::new();
        let _ = detector.start();

        // イベントがない場合はfalseを返す
        let has_event = detector.handle_events();
        assert!(!has_event);

        detector.stop();
    }

    #[test]
    fn test_edge_detector_stop() {
        let mut detector = EdgeDetector::new();
        #[cfg(target_os = "windows")]
        {
            let _ = detector.start();
        }

        detector.stop();
        assert!(!detector.running.load(Ordering::Relaxed));
        assert!(detector.sender.is_none());
        assert!(detector.receiver.is_none());
        assert!(detector.thread_handle.is_none());
    }

    #[test]
    fn test_edge_detector_stop_when_not_running() {
        let mut detector = EdgeDetector::new();
        // 起動していない状態でstopを呼んでもパニックしない
        detector.stop();
        assert!(!detector.running.load(Ordering::Relaxed));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_screen_width() {
        let width = get_screen_width();
        // 画面幅は正の値であるべき
        assert!(width > 0);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_cursor_pos() {
        let result = get_cursor_pos();
        // カーソル位置の取得は成功するはず
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_detector_default() {
        let detector = EdgeDetector::default();
        assert!(!detector.running.load(Ordering::Relaxed));
    }

    #[test]
    fn test_edge_detector_drop() {
        let mut detector = EdgeDetector::new();
        #[cfg(target_os = "windows")]
        {
            let _ = detector.start();
        }

        // スコープを抜けるとDropが呼ばれてスレッドが停止する
        drop(detector);

        // Dropが正常に動作することを確認（パニックしない）
        // この時点でdetectorは既にドロップされている
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_edge_detector_lifecycle() {
        // 完全なライフサイクルテスト
        let mut detector = EdgeDetector::new();

        // 初期状態
        assert!(!detector.running.load(Ordering::Relaxed));

        // 起動
        assert!(detector.start().is_ok());
        assert!(detector.running.load(Ordering::Relaxed));

        // 少し待機
        thread::sleep(Duration::from_millis(100));

        // イベント処理
        let _ = detector.handle_events();

        // 停止
        detector.stop();
        assert!(!detector.running.load(Ordering::Relaxed));
    }
}
