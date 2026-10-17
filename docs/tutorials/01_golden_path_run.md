# Tutorial: The Golden Path Run

**Difficulty:** Beginner  
**Estimated Time:** 5 minutes  

---

## 1. Prerequisites

Ensure `affi` is built and present on your path:
```bash
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

---

## 2. Step 1: Record an Event (`emit`)

Record an event in the local provenance journal:
```bash
affi emit --event-type "build.completed" --payload '{"status":"ok","tests":42}'
```

---

## 3. Step 2: Assemble and Seal a Receipt (`assemble`)

Finalize the current operation-event buffer into a sealed, content-addressed receipt:
```bash
affi assemble --output receipt.json
```

---

## 4. Step 3: Verify the Receipt (`verify`)

Execute the 7-stage certify pipeline against the sealed receipt:
```bash
affi verify receipt.json
```
Output:
```text
VERDICT: ACCEPT
Chain Hash: [32-byte BLAKE3 Root]
Stages Passed: 7/7
```
