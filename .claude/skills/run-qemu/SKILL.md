---
name: run-qemu
description: MikanOS を QEMU で起動する。ブートローダー EFI とカーネル ELF を渡して run_qemu.sh を実行する。
---

QEMU で MikanOS を起動してください：

```bash
~/osbook/devenv/run_qemu.sh ~/edk2/Build/MikanLoaderX64/DEBUG_CLANG38/X64/Loader.efi /workspace/mikanos/kernel/kernel.elf
```

起動したら QEMU ウィンドウが開いたことをユーザーに伝え、
終了するには QEMU ウィンドウを閉じるか、「QEMU を終了して」と伝えるよう案内してください。

ユーザーから終了を求められたら TaskStop ツールでバックグラウンドタスクを停止してください。

エラーが出た場合はエラーメッセージを表示してください。
