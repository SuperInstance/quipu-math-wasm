//! # quipu-math-wasm
//!
//! Incan knotted cord (quipu) encoding compiled to WebAssembly.
//!
//! Quipu are Incan knotted cord recording devices. This crate implements the
//! mathematics of knotted data structures: encoding numbers as knot sequences,
//! building cord hierarchities, arithmetic on quipus, error detection via parity
//! knots, and weave/unweave operations.

#![deny(unsafe_code)]

use wasm_bindgen::prelude::*;

// ── Knot types ──────────────────────────────────────────────────────────

/// Types of knots used in quipu encoding.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KnotType {
    /// A single knot — encodes digit 1 in non-units positions.
    Single,
    /// A figure-eight knot — encodes digit 1 in the units position.
    FigureEight,
    /// A long knot with multiple turns (2–9) — encodes digits 2–9.
    Long,
}

/// A single knot on a cord.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knot {
    /// The type of knot.
    pub knot_type: KnotType,
    /// Number of turns (meaningful only for long knots, values 2–9).
    pub value: u8,
    /// Decimal position (0 = units, 1 = tens, 2 = hundreds, …).
    pub position: u32,
}

#[wasm_bindgen]
impl Knot {
    /// Create a new knot.
    #[wasm_bindgen(constructor)]
    pub fn new(knot_type: KnotType, value: u8, position: u32) -> Knot {
        Knot {
            knot_type,
            value,
            position,
        }
    }

    /// The decimal digit this knot represents.
    pub fn digit_value(&self) -> u32 {
        match self.knot_type {
            KnotType::FigureEight => 1,
            KnotType::Single => 1,
            KnotType::Long => self.value as u32,
        }
    }
}

// ── Encode / decode ────────────────────────────────────────────────────

/// Encode a non-negative integer as a sequence of quipu knots.
///
/// Each digit maps to:
/// - 0 → no knot at that position
/// - 1 → figure_eight (units, position 0) or single (position > 0)
/// - 2–9 → long knot with that many turns
#[wasm_bindgen]
pub fn encode_number(n: u32) -> Vec<Knot> {
    if n == 0 {
        return Vec::new();
    }

    let mut digits = Vec::new();
    let mut val = n;
    while val > 0 {
        digits.push((val % 10) as u8);
        val /= 10;
    }

    let mut knots = Vec::new();
    for (position, &digit) in digits.iter().enumerate() {
        if digit == 0 {
            continue;
        } else if digit == 1 {
            if position == 0 {
                knots.push(Knot {
                    knot_type: KnotType::FigureEight,
                    value: 1,
                    position: position as u32,
                });
            } else {
                knots.push(Knot {
                    knot_type: KnotType::Single,
                    value: 1,
                    position: position as u32,
                });
            }
        } else {
            knots.push(Knot {
                knot_type: KnotType::Long,
                value: digit,
                position: position as u32,
            });
        }
    }
    knots
}

/// Decode a knot sequence back to an integer.
#[wasm_bindgen]
pub fn decode_number(knots: Vec<Knot>) -> u32 {
    let mut total: u32 = 0;
    for knot in &knots {
        total += knot.digit_value() * 10u32.pow(knot.position);
    }
    total
}

/// Compute a simple checksum (sum of digit values mod 10) for a knot sequence.
/// This implements Incan-style error detection.
#[wasm_bindgen]
pub fn checksum(knots: Vec<Knot>) -> u32 {
    let sum: u32 = knots.iter().map(|k| k.digit_value()).sum();
    sum % 10
}

// ── CordTree ────────────────────────────────────────────────────────────

/// A pendant cord in a cord tree.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PendantCord {
    color: u32,
    knots: Vec<Knot>,
}

#[wasm_bindgen]
impl PendantCord {
    #[wasm_bindgen(constructor)]
    pub fn new(color: u32) -> PendantCord {
        PendantCord {
            color,
            knots: Vec::new(),
        }
    }

    pub fn color(&self) -> u32 {
        self.color
    }

    /// The numeric value encoded in this cord's knots.
    pub fn value(&self) -> u32 {
        decode_number(self.knots.clone())
    }

    /// Get the knots on this pendant.
    pub fn knots(&self) -> Vec<Knot> {
        self.knots.clone()
    }
}

/// A full quipu: main cord with pendant cords.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct CordTree {
    pendants: Vec<PendantCord>,
}

#[wasm_bindgen]
impl CordTree {
    /// Create a new empty cord tree.
    #[wasm_bindgen(constructor)]
    pub fn new() -> CordTree {
        CordTree {
            pendants: Vec::new(),
        }
    }

    /// Add a pendant cord with the given color.
    pub fn add_pendant(&mut self, color: u32) {
        self.pendants.push(PendantCord {
            color,
            knots: Vec::new(),
        });
    }

    /// Encode a value onto a specific pendant cord.
    pub fn encode_on_pendant(&mut self, pendant_idx: usize, value: u32) {
        if pendant_idx < self.pendants.len() {
            self.pendants[pendant_idx].knots = encode_number(value);
        }
    }

    /// Decode the value from a specific pendant cord.
    pub fn decode_pendant(&self, pendant_idx: usize) -> u32 {
        if pendant_idx < self.pendants.len() {
            self.pendants[pendant_idx].value()
        } else {
            0
        }
    }

    /// Sum of all pendant cord values.
    pub fn total_value(&self) -> u32 {
        self.pendants.iter().map(|p| p.value()).sum()
    }

    /// Number of pendant cords.
    pub fn pendant_count(&self) -> usize {
        self.pendants.len()
    }

    /// Get the pendant count as u32 (for wasm-bindgen convenience).
    pub fn pendant_count_u32(&self) -> u32 {
        self.pendants.len() as u32
    }
}

// ── Quipu arithmetic ───────────────────────────────────────────────────

/// Add two quipu trees element-wise (pendant by pendant).
///
/// If the trees have different numbers of pendants, the shorter is zero-padded.
/// The result has one pendant per position whose value is the sum.
#[wasm_bindgen]
pub fn quipu_add(a: &CordTree, b: &CordTree) -> CordTree {
    let max_len = a.pendants.len().max(b.pendants.len());
    let mut result = CordTree::new();
    for i in 0..max_len {
        let va = if i < a.pendants.len() {
            a.pendants[i].value()
        } else {
            0
        };
        let vb = if i < b.pendants.len() {
            b.pendants[i].value()
        } else {
            0
        };
        let mut cord = PendantCord::new(0x008000); // green
        cord.knots = encode_number(va + vb);
        result.pendants.push(cord);
    }
    result
}

/// Subtract tree b from tree a element-wise.
///
/// Panics if any result would be negative.
#[wasm_bindgen]
pub fn quipu_subtract(a: &CordTree, b: &CordTree) -> CordTree {
    let max_len = a.pendants.len().max(b.pendants.len());
    let mut result = CordTree::new();
    for i in 0..max_len {
        let va = if i < a.pendants.len() {
            a.pendants[i].value()
        } else {
            0
        };
        let vb = if i < b.pendants.len() {
            b.pendants[i].value()
        } else {
            0
        };
        assert!(va >= vb, "Negative result at position {}: {} - {}", i, va, vb);
        let mut cord = PendantCord::new(0x0000FF); // blue
        cord.knots = encode_number(va - vb);
        result.pendants.push(cord);
    }
    result
}

// ── Corruption detection ───────────────────────────────────────────────

/// Report from comparing two cord trees for corruption.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CorruptionReport {
    /// True if all pendant values match.
    pub value_ok: bool,
    /// True if pendant counts match.
    pub count_ok: bool,
    /// True if all parity checks match.
    pub parity_ok: bool,
}

#[wasm_bindgen]
impl CorruptionReport {
    #[wasm_bindgen(constructor)]
    pub fn new(value_ok: bool, count_ok: bool, parity_ok: bool) -> CorruptionReport {
        CorruptionReport {
            value_ok,
            count_ok,
            parity_ok,
        }
    }
}

/// Compare two cord trees and detect corruption.
///
/// Checks:
/// - Pendant count matches
/// - Value matches at each position
/// - Parity (checksum mod 10) matches at each position
#[wasm_bindgen]
pub fn detect_corruption(original: &CordTree, copy: &CordTree) -> CorruptionReport {
    let count_ok = original.pendants.len() == copy.pendants.len();

    let min_len = original.pendants.len().min(copy.pendants.len());
    let mut value_ok = true;
    let mut parity_ok = true;

    for i in 0..min_len {
        if original.pendants[i].value() != copy.pendants[i].value() {
            value_ok = false;
        }
        let orig_parity = checksum(original.pendants[i].knots.clone());
        let copy_parity = checksum(copy.pendants[i].knots.clone());
        if orig_parity != copy_parity {
            parity_ok = false;
        }
    }

    // If counts differ, values can't all be ok
    if original.pendants.len() != copy.pendants.len() {
        value_ok = false;
    }

    CorruptionReport {
        value_ok,
        count_ok,
        parity_ok,
    }
}

// ── Weave / Unweave ────────────────────────────────────────────────────

/// Weave two u32 values into a single u64.
///
/// Encodes the pair (v1, v2) as v1 * 10000 + v2 in the lower 32 bits
/// and stores v1 in the upper 32 bits for lossless recovery.
/// Actually: combined = (v1 as u64) << 32 | (v2 as u64)
#[wasm_bindgen]
pub fn weave(v1: u32, v2: u32) -> u64 {
    ((v1 as u64) << 32) | (v2 as u64)
}

/// Unweave a combined u64 back into its two u32 components.
///
/// Returns the two values as a flat array [v1, v2].
#[wasm_bindgen]
pub fn unweave(woven: u64) -> Vec<u32> {
    let v1 = (woven >> 32) as u32;
    let v2 = (woven & 0xFFFFFFFF) as u32;
    vec![v1, v2]
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Knot basics ─────────────────────────────────────────────────

    #[test]
    fn test_knot_digit_value_figure_eight() {
        let k = Knot::new(KnotType::FigureEight, 1, 0);
        assert_eq!(k.digit_value(), 1);
    }

    #[test]
    fn test_knot_digit_value_single() {
        let k = Knot::new(KnotType::Single, 1, 1);
        assert_eq!(k.digit_value(), 1);
    }

    #[test]
    fn test_knot_digit_value_long() {
        let k = Knot::new(KnotType::Long, 7, 0);
        assert_eq!(k.digit_value(), 7);
    }

    #[test]
    fn test_knot_equality() {
        let k1 = Knot::new(KnotType::Long, 5, 1);
        let k2 = Knot::new(KnotType::Long, 5, 1);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_knot_type_equality() {
        assert_eq!(KnotType::Single, KnotType::Single);
        assert_ne!(KnotType::Single, KnotType::Long);
    }

    // ── Encode ──────────────────────────────────────────────────────

    #[test]
    fn test_encode_zero() {
        let knots = encode_number(0);
        assert!(knots.is_empty());
    }

    #[test]
    fn test_encode_one() {
        let knots = encode_number(1);
        assert_eq!(knots.len(), 1);
        assert_eq!(knots[0].knot_type, KnotType::FigureEight);
        assert_eq!(knots[0].position, 0);
    }

    #[test]
    fn test_encode_nine() {
        let knots = encode_number(9);
        assert_eq!(knots.len(), 1);
        assert_eq!(knots[0].knot_type, KnotType::Long);
        assert_eq!(knots[0].value, 9);
    }

    #[test]
    fn test_encode_ten() {
        let knots = encode_number(10);
        assert_eq!(knots.len(), 1);
        assert_eq!(knots[0].knot_type, KnotType::Single);
        assert_eq!(knots[0].position, 1);
    }

    #[test]
    fn test_encode_247() {
        let knots = encode_number(247);
        assert_eq!(knots.len(), 3);
        // digits: 7 (units=0), 4 (tens=1), 2 (hundreds=2)
        let by_pos: std::collections::HashMap<u32, &Knot> =
            knots.iter().map(|k| (k.position, k)).collect();
        assert_eq!(by_pos[&0].knot_type, KnotType::Long);
        assert_eq!(by_pos[&0].value, 7);
        assert_eq!(by_pos[&1].knot_type, KnotType::Long);
        assert_eq!(by_pos[&1].value, 4);
        assert_eq!(by_pos[&2].knot_type, KnotType::Long);
        assert_eq!(by_pos[&2].value, 2);
    }

    #[test]
    fn test_encode_1001() {
        let knots = encode_number(1001);
        assert_eq!(knots.len(), 2);
        let by_pos: std::collections::HashMap<u32, &Knot> =
            knots.iter().map(|k| (k.position, k)).collect();
        // 1 in units → figure_eight, 1 in thousands → single
        assert_eq!(by_pos[&0].knot_type, KnotType::FigureEight);
        assert_eq!(by_pos[&3].knot_type, KnotType::Single);
    }

    // ── Decode ──────────────────────────────────────────────────────

    #[test]
    fn test_decode_empty() {
        assert_eq!(decode_number(vec![]), 0);
    }

    #[test]
    fn test_roundtrip_single_digits() {
        for n in 1u32..10 {
            assert_eq!(decode_number(encode_number(n)), n);
        }
    }

    #[test]
    fn test_roundtrip_various() {
        for &n in &[0u32, 1, 10, 42, 100, 247, 1000, 9999] {
            assert_eq!(decode_number(encode_number(n)), n, "roundtrip failed for {}", n);
        }
    }

    // ── Checksum ────────────────────────────────────────────────────

    #[test]
    fn test_checksum_empty() {
        assert_eq!(checksum(vec![]), 0);
    }

    #[test]
    fn test_checksum_247() {
        // encode_number(247) → digits 7, 4, 2 → sum = 13 → 13 % 10 = 3
        let knots = encode_number(247);
        assert_eq!(checksum(knots), 3);
    }

    #[test]
    fn test_checksum_single_digit() {
        let knots = encode_number(5);
        assert_eq!(checksum(knots), 5);
    }

    // ── CordTree ────────────────────────────────────────────────────

    #[test]
    fn test_cord_tree_empty() {
        let tree = CordTree::new();
        assert_eq!(tree.pendant_count(), 0);
        assert_eq!(tree.total_value(), 0);
    }

    #[test]
    fn test_cord_tree_add_pendant() {
        let mut tree = CordTree::new();
        tree.add_pendant(0xFF0000); // red
        tree.add_pendant(0x0000FF); // blue
        assert_eq!(tree.pendant_count(), 2);
    }

    #[test]
    fn test_cord_tree_encode_decode_pendant() {
        let mut tree = CordTree::new();
        tree.add_pendant(0xFF0000);
        tree.encode_on_pendant(0, 42);
        assert_eq!(tree.decode_pendant(0), 42);
    }

    #[test]
    fn test_cord_tree_total_value() {
        let mut tree = CordTree::new();
        tree.add_pendant(0xFF0000);
        tree.encode_on_pendant(0, 100);
        tree.add_pendant(0x0000FF);
        tree.encode_on_pendant(1, 47);
        assert_eq!(tree.total_value(), 147);
    }

    #[test]
    fn test_cord_tree_decode_out_of_bounds() {
        let tree = CordTree::new();
        assert_eq!(tree.decode_pendant(0), 0);
    }

    #[test]
    fn test_cord_tree_encode_out_of_bounds() {
        let mut tree = CordTree::new();
        tree.encode_on_pendant(0, 42); // should be no-op
        assert_eq!(tree.pendant_count(), 0);
    }

    #[test]
    fn test_pendant_cord_value() {
        let mut pc = PendantCord::new(0xFF0000);
        pc.knots = encode_number(99);
        assert_eq!(pc.value(), 99);
        assert_eq!(pc.color(), 0xFF0000);
    }

    // ── Quipu arithmetic ────────────────────────────────────────────

    #[test]
    fn test_quipu_add() {
        let mut a = CordTree::new();
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(0, 10);
        let mut b = CordTree::new();
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(0, 25);
        let result = quipu_add(&a, &b);
        assert_eq!(result.pendant_count(), 1);
        assert_eq!(result.decode_pendant(0), 35);
    }

    #[test]
    fn test_quipu_add_multiple() {
        let mut a = CordTree::new();
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(0, 10);
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(1, 20);
        let mut b = CordTree::new();
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(0, 5);
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(1, 15);
        let result = quipu_add(&a, &b);
        assert_eq!(result.decode_pendant(0), 15);
        assert_eq!(result.decode_pendant(1), 35);
    }

    #[test]
    fn test_quipu_subtract() {
        let mut a = CordTree::new();
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(0, 50);
        let mut b = CordTree::new();
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(0, 20);
        let result = quipu_subtract(&a, &b);
        assert_eq!(result.decode_pendant(0), 30);
    }

    #[test]
    #[should_panic(expected = "Negative result")]
    fn test_quipu_subtract_negative_panics() {
        let mut a = CordTree::new();
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(0, 5);
        let mut b = CordTree::new();
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(0, 10);
        let _ = quipu_subtract(&a, &b);
    }

    #[test]
    fn test_quipu_add_different_lengths() {
        let mut a = CordTree::new();
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(0, 10);
        a.add_pendant(0xFFFFFF);
        a.encode_on_pendant(1, 20);
        let mut b = CordTree::new();
        b.add_pendant(0xFFFFFF);
        b.encode_on_pendant(0, 5);
        let result = quipu_add(&a, &b);
        assert_eq!(result.pendant_count(), 2);
        assert_eq!(result.decode_pendant(0), 15);
        assert_eq!(result.decode_pendant(1), 20);
    }

    // ── Corruption detection ────────────────────────────────────────

    #[test]
    fn test_detect_no_corruption() {
        let mut tree1 = CordTree::new();
        tree1.add_pendant(0xFF0000);
        tree1.encode_on_pendant(0, 42);
        let mut tree2 = CordTree::new();
        tree2.add_pendant(0xFF0000);
        tree2.encode_on_pendant(0, 42);
        let report = detect_corruption(&tree1, &tree2);
        assert!(report.value_ok);
        assert!(report.count_ok);
        assert!(report.parity_ok);
    }

    #[test]
    fn test_detect_value_corruption() {
        let mut tree1 = CordTree::new();
        tree1.add_pendant(0xFF0000);
        tree1.encode_on_pendant(0, 10);
        let mut tree2 = CordTree::new();
        tree2.add_pendant(0xFF0000);
        tree2.encode_on_pendant(0, 20);
        let report = detect_corruption(&tree1, &tree2);
        assert!(!report.value_ok);
        assert!(report.count_ok);
    }

    #[test]
    fn test_detect_count_corruption() {
        let mut tree1 = CordTree::new();
        tree1.add_pendant(0xFF0000);
        tree1.encode_on_pendant(0, 10);
        let mut tree2 = CordTree::new();
        tree2.add_pendant(0xFF0000);
        tree2.encode_on_pendant(0, 10);
        tree2.add_pendant(0x0000FF);
        tree2.encode_on_pendant(1, 5);
        let report = detect_corruption(&tree1, &tree2);
        assert!(!report.count_ok);
        assert!(!report.value_ok); // counts differ → values can't all match
    }

    #[test]
    fn test_detect_parity_corruption() {
        // Two different values that might coincidentally have different parity
        let mut tree1 = CordTree::new();
        tree1.add_pendant(0xFF0000);
        tree1.encode_on_pendant(0, 7); // digit sum = 7, parity = 7
        let mut tree2 = CordTree::new();
        tree2.add_pendant(0xFF0000);
        tree2.encode_on_pendant(0, 8); // digit sum = 8, parity = 8
        let report = detect_corruption(&tree1, &tree2);
        assert!(!report.parity_ok);
        assert!(!report.value_ok);
    }

    #[test]
    fn test_corruption_report_constructor() {
        let report = CorruptionReport::new(true, false, true);
        assert!(report.value_ok);
        assert!(!report.count_ok);
        assert!(report.parity_ok);
    }

    #[test]
    fn test_corruption_report_equality() {
        let r1 = CorruptionReport::new(true, true, false);
        let r2 = CorruptionReport::new(true, true, false);
        assert_eq!(r1, r2);
    }

    // ── Weave / Unweave ─────────────────────────────────────────────

    #[test]
    fn test_weave_basic() {
        let woven = weave(3, 7);
        let parts = unweave(woven);
        assert_eq!(parts[0], 3);
        assert_eq!(parts[1], 7);
    }

    #[test]
    fn test_weave_roundtrip() {
        let woven = weave(42, 13);
        let parts = unweave(woven);
        assert_eq!(parts[0], 42);
        assert_eq!(parts[1], 13);
    }

    #[test]
    fn test_weave_zero() {
        let woven = weave(0, 0);
        assert_eq!(woven, 0);
        let parts = unweave(0);
        assert_eq!(parts[0], 0);
        assert_eq!(parts[1], 0);
    }

    #[test]
    fn test_weave_max() {
        let woven = weave(u32::MAX, u32::MAX);
        let parts = unweave(woven);
        assert_eq!(parts[0], u32::MAX);
        assert_eq!(parts[1], u32::MAX);
    }

    #[test]
    fn test_weave_large_values() {
        let woven = weave(9999, 12345);
        let parts = unweave(woven);
        assert_eq!(parts[0], 9999);
        assert_eq!(parts[1], 12345);
    }

    // ── Pendant cord ────────────────────────────────────────────────

    #[test]
    fn test_pendant_knots() {
        let mut pc = PendantCord::new(0x00FF00);
        pc.knots = encode_number(55);
        assert_eq!(pc.knots().len(), 2); // 5 at position 0, 5 at position 1
    }

    #[test]
    fn test_pendant_empty_value() {
        let pc = PendantCord::new(0xFFFFFF);
        assert_eq!(pc.value(), 0);
    }

    // ── Large number roundtrip ──────────────────────────────────────

    #[test]
    fn test_large_number_roundtrip() {
        let n = 123456u32;
        assert_eq!(decode_number(encode_number(n)), n);
    }

    #[test]
    fn test_max_u32_small() {
        // Test with a value that fits well
        for &n in &[99u32, 999, 9999, 99999] {
            assert_eq!(decode_number(encode_number(n)), n);
        }
    }

    // ── Corruption with multiple pendants ───────────────────────────

    #[test]
    fn test_detect_corruption_multi_pendant() {
        let mut orig = CordTree::new();
        orig.add_pendant(0xFF0000);
        orig.encode_on_pendant(0, 100);
        orig.add_pendant(0x0000FF);
        orig.encode_on_pendant(1, 200);

        let mut copy = CordTree::new();
        copy.add_pendant(0xFF0000);
        copy.encode_on_pendant(0, 100);
        copy.add_pendant(0x0000FF);
        copy.encode_on_pendant(1, 200);

        let report = detect_corruption(&orig, &copy);
        assert!(report.value_ok);
        assert!(report.count_ok);
        assert!(report.parity_ok);
    }

    #[test]
    fn test_detect_corruption_partial_mismatch() {
        let mut orig = CordTree::new();
        orig.add_pendant(0xFF0000);
        orig.encode_on_pendant(0, 100);
        orig.add_pendant(0x0000FF);
        orig.encode_on_pendant(1, 200);

        let mut copy = CordTree::new();
        copy.add_pendant(0xFF0000);
        copy.encode_on_pendant(0, 100);
        copy.add_pendant(0x0000FF);
        copy.encode_on_pendant(1, 201); // corrupted

        let report = detect_corruption(&orig, &copy);
        assert!(!report.value_ok);
    }
}
