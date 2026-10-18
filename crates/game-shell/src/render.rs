//! 渲染合成（调用 `game_platform::Renderer` 的 256×240 RGBA 画布；v1 显式占位）。
//!
//! Stage A 纪律（未证不发明）：v1 **只**画「已证坐标面」的玩家占位块——
//! `player.x/y`（`$04E0/$04FC`，A 级）→ 8×8 方块。**图样/调色板=占位，非原版像素**。
//!
//! `TODO(platform-v1)`：以下渲染域一律**不画**（未证或属后续渲染战役）：
//! BG/nametable、敌精灵、弹丸（H-01）、HUD、文本流、结局卡、fade/调色板影子上传、
//! 敌人槽/子弹槽可视化；像素级一致（L4）不是本层目标。
//!
//! 实现侧（`shell-pc`/`shell-android`）负责把本画布整数倍缩放/黑边/上屏；
//! 本模块只写逻辑画布并返回确定性画布 hash（宿主签名用）。

use game_core::flow::Mode;
use game_core::player::PlayerState;
use game_platform::{CANVAS_H, CANVAS_W};

/// FNV-1a 64 偏移基。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64 素数。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 玩家占位块颜色（**占位**：非原版图样/调色板，仅表达已证坐标）。
pub const PLACEHOLDER_PLAYER_RGBA: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

/// 合成视图（平台无关；只含宿主已证/已接线状态，不含未证渲染数据）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameView {
    /// 逻辑帧号。
    pub frame: u64,
    /// flow 模式（v1 不用于绘制；后续模式化界面接线时使用）。
    pub mode: Mode,
    /// 关号（`$180`，1..=8）。
    pub stage: u8,
    /// 玩家状态（唯一 v1 绘制源：已证 x/y 坐标面）。
    pub player: PlayerState,
    /// 活弹数（engine 已暴露面；v1 **不画**，仅登记供后续渲染战役）。
    pub live_bullets: u16,
}

/// 画布 hash（FNV-1a 64，逐字节）。
pub fn canvas_hash(canvas: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in canvas {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// 清屏：RGBA 全填黑（不透明）；尾部非 RGBA 对齐字节清 0（契约外防御）。
fn clear_canvas(canvas: &mut [u8]) {
    for px in canvas.chunks_exact_mut(4) {
        px.copy_from_slice(&[0, 0, 0, 0xFF]);
    }
    let rem = canvas.len() % 4;
    if rem != 0 {
        let start = canvas.len() - rem;
        canvas[start..].fill(0);
    }
}

/// 玩家占位块（8×8，越界裁剪；坐标取已证 `$04E0/$04FC`，图样=占位）。
fn draw_player_marker(canvas: &mut [u8], px: i32, py: i32) {
    for dy in 0..8i32 {
        for dx in 0..8i32 {
            let x = px + dx;
            let y = py + dy;
            if x < 0 || y < 0 || x >= CANVAS_W as i32 || y >= CANVAS_H as i32 {
                continue;
            }
            let off = (y as usize * CANVAS_W + x as usize) * 4;
            if off + 4 <= canvas.len() {
                canvas[off..off + 4].copy_from_slice(&PLACEHOLDER_PLAYER_RGBA);
            }
        }
    }
}

/// 一帧合成（清屏 + 玩家占位块），返回画布 hash（确定性签名用）。
///
/// 画布尺寸契约=256×240 RGBA；若实现返回更小画布，绘制按实际长度 fail-closed 裁剪
/// （不 panic——平台实现错误不应拖垮逻辑帧）。
pub fn compose_frame(canvas: &mut [u8], view: &FrameView) -> u64 {
    clear_canvas(canvas);
    draw_player_marker(canvas, i32::from(view.player.x), i32::from(view.player.y));
    canvas_hash(canvas)
}

#[cfg(test)]
mod tests {
    use super::{canvas_hash, compose_frame, FrameView, PLACEHOLDER_PLAYER_RGBA};
    use game_core::flow::Mode;
    use game_core::player::PlayerState;
    use game_platform::CANVAS_LEN;

    fn view(player: PlayerState) -> FrameView {
        FrameView { frame: 0, mode: Mode::Combat, stage: 1, player, live_bullets: 0 }
    }

    fn pixel(canvas: &[u8], x: usize, y: usize) -> [u8; 4] {
        let o = (y * 256 + x) * 4;
        [canvas[o], canvas[o + 1], canvas[o + 2], canvas[o + 3]]
    }

    #[test]
    fn compose_is_deterministic_and_draws_player_at_proven_coords() {
        let player = PlayerState { x: 10, y: 20, ..PlayerState::default() };
        let mut a = vec![0u8; CANVAS_LEN];
        let mut b = vec![0u8; CANVAS_LEN];
        let sig_a = compose_frame(&mut a, &view(player));
        let sig_b = compose_frame(&mut b, &view(player));
        assert_eq!(sig_a, sig_b, "同视图⇒同画布 hash");
        assert_eq!(a, b, "同视图⇒同画布字节");
        assert_eq!(pixel(&a, 10, 20), PLACEHOLDER_PLAYER_RGBA, "玩家块起点=已证坐标");
        assert_eq!(pixel(&a, 17, 27), PLACEHOLDER_PLAYER_RGBA, "玩家块 8×8");
        assert_eq!(pixel(&a, 9, 20), [0, 0, 0, 0xFF], "块外=黑底");
        assert_eq!(pixel(&a, 0, 0), [0, 0, 0, 0xFF], "清屏=不透明黑");
    }

    #[test]
    fn player_marker_clips_at_edges() {
        let player = PlayerState { x: 255, y: 239, ..PlayerState::default() };
        let mut canvas = vec![0u8; CANVAS_LEN];
        compose_frame(&mut canvas, &view(player));
        assert_eq!(pixel(&canvas, 255, 239), PLACEHOLDER_PLAYER_RGBA, "右下角像素在画布内");
        assert_eq!(pixel(&canvas, 254, 239), [0, 0, 0, 0xFF], "块起点左侧=块外（越界裁剪）");
    }

    #[test]
    fn canvas_depends_only_on_player_coords_in_v1() {
        let base = PlayerState { x: 128, y: 160, ..PlayerState::default() };
        let mut a = vec![0u8; CANVAS_LEN];
        let mut b = vec![0u8; CANVAS_LEN];
        compose_frame(&mut a, &view(base));
        let mut view_b = view(base);
        view_b.frame = 999;
        view_b.mode = Mode::Ending;
        view_b.stage = 8;
        view_b.live_bullets = 77;
        compose_frame(&mut b, &view_b);
        assert_eq!(a, b, "v1 画布只由已证玩家坐标决定（其余字段不画）");
    }

    #[test]
    fn canvas_hash_detects_changes() {
        let mut a = vec![0u8; CANVAS_LEN];
        let mut b = vec![0u8; CANVAS_LEN];
        let ha = compose_frame(&mut a, &view(PlayerState { x: 1, ..PlayerState::default() }));
        let hb = compose_frame(&mut b, &view(PlayerState { x: 2, ..PlayerState::default() }));
        assert_ne!(ha, hb, "不同画布⇒不同 hash");
        assert_ne!(canvas_hash(&[0u8; 8]), canvas_hash(&[1u8; 8]));
    }
}
