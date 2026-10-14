# reference/jp — JP 原版参照锚（D3 双轨：参照锚，非主锚）

- 文件: gunnac_jp.nes（262,160B，SHA-256 08ead6a2…111da44d = zno5/gun-nac-ko config/rom.json 社区标准）
- 来源: 用户提供 "Gun Nac (Japan).zip"（zip sha256 7e7af025…，内含 NES 2.0 头 dump sha256 6c107f9f…）
- 处置: 本体（PRG+CHR，260KB）与社区标准**逐位一致**（经典 iNES 头重建后 md5 029daa68… 精确命中）；入库文件头**规范化为经典 iNES**（flags6=0x40/flags7=0x00）以便与社区 hash 对齐
- 角色: **参照锚**——JP↔九班差分基准 + 社区成果核验基准。主锚仍为 rom/gunnac_cn.nes（D3 不变）
