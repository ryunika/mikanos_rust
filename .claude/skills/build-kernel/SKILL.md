---
name: build-kernel
description: MikanOS カーネル（Rust）をビルドする。/workspace/mikanos/kernel で cargo build を実行し、生成された ELF を kernel.elf としてコピーする。
---

/workspace/mikanos/kernel でカーネルをビルドしてください：

```bash
cd /workspace/mikanos/kernel && cargo build
cp target/x86_64-unknown-none-elf/debug/kernel kernel.elf
```

ビルドが成功したら kernel.elf のパスをユーザーに伝えてください。
エラーが出た場合はコンパイルエラーの内容を表示してください。
