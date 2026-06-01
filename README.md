# quipu-math-wasm

> Incan knotted cord mathematics compiled to WebAssembly.

## What This Does

`quipu-math-wasm` brings the Incan quipu number system to the browser at near-native speed. Encode/decode numbers as knots, perform arithmetic on quipu trees, detect corruption, and weave/unweave cords. Use it for educational web tools, novel encoding demonstrations, or cultural math exploration.

## The Cultural Root

See `quipu-math` (PyPI) for the full cultural background. Quipu encode decimal numbers as knotted cords — a tactile positional system.

## Install

```bash
npm install quipu-math-wasm
```

## Quick Start

```typescript
import init, {
  encode_number, decode_number, checksum,
  CordTree, quipu_add, quipu_subtract,
  weave, unweave, detect_corruption,
} from "quipu-math-wasm";

await init();

// Encode and decode
const knots = encode_number(135);
console.log(decode_number(knots));  // 135
console.log(checksum(knots));       // 9

// Build quipu tree
const tree = new CordTree();
tree.add_pendant("red", 42);
tree.add_pendant("blue", 17);

// Arithmetic
const tree2 = new CordTree();
tree2.add_pendant("green", 10);
const sum = quipu_add(tree, tree2);

// Weaving
const woven = weave(7, 3);
const [a, b] = unweave(woven);

// Corruption detection
const report = detect_corruption(tree, tree2);
console.log(report.corrupted);
```

## API Reference

### Encoding
- `encode_number(n: number) → Knot[]`
- `decode_number(knots: Knot[]) → number`
- `checksum(knots: Knot[]) → number`

### `CordTree`
- `add_pendant(color: string, value: number)`
- `total_value() → number`

### Arithmetic
- `quipu_add(a: CordTree, b: CordTree) → CordTree`
- `quipu_subtract(a: CordTree, b: CordTree) → CordTree`

### Weaving
- `weave(v1: number, v2: number) → bigint`
- `unweave(woven: bigint) → number[]`

### Corruption
- `detect_corruption(original: CordTree, copy: CordTree) → CorruptionReport`

## License

MIT
