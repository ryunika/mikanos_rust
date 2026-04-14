---
name: run-qemu
description: MikanOS を QEMU で起動する。ブートローダー EFI とカーネル ELF を渡して run_qemu.sh を実行する。
---

QEMU で MikanOS を起動してください：

```bash
~/osbook/devenv/run_qemu.sh ~/edk2/Build/MikanLoaderX64/DEBUG_CLANG38/X64/Loader.efi /workspace/mikanos/kernel/kernel.elf
```

このコマンドはバックグラウンドで実行してください。

起動したら QEMU ウィンドウが開いたことをユーザーに伝え、
終了するには QEMU ウィンドウを閉じるか、「QEMU を終了して」と伝えるよう案内してください。

## 注意事項

- `run_qemu.sh` はシェルスクリプトであり、スクリプト自体はすぐ終了しますが、
  内部で起動した `qemu-system-x86_64` プロセスは独立して動き続けます。
- スクリプトの終了（exit code 0）を QEMU の終了と誤解しないこと。

## 終了方法

ユーザーから終了を求められたら、以下の手順で終了してください：

1. `pgrep -a qemu` で実行中の QEMU プロセスを確認する
2. `kill <PID>` で `qemu-system-x86_64` プロセスを終了する
3. 終了を確認してユーザーに報告する

エラーが出た場合はエラーメッセージを表示してください。
