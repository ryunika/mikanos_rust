---
name: build-bootloader
description: MikanOS ブートローダー（UEFI/EDK2）をビルドする。~/edk2 で EDK2 の build コマンドを実行する。
---

EDK2 を使ってブートローダーをビルドしてください：

```bash
cd ~/edk2 && source edksetup.sh && build
```

ビルドが成功したら出力先 `~/edk2/Build/MikanLoaderX64/DEBUG_CLANG38/X64/Loader.efi` をユーザーに伝えてください。
エラーが出た場合はエラーメッセージを表示してください。
