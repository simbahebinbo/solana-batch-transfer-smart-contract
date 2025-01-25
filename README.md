# batch-transfer

批量转账/空投合约

该合约用来从一个地址往多个地址转币

1、能批量转账sol本币和spl代币；
2、每次使用，收取手续费，手续费可以调整；

```shell
$ cargo version
cargo 1.84.0 (66221abde 2024-11-19)
$ rustc --version
rustc 1.84.0 (9fc6b4312 2025-01-07)
```

```shell
$ solana --version
solana-cli 2.0.22 (src:faea52f3; feat:607245837, client:Agave)
```

```shell
$ solana-test-validator --version
solana-test-validator 2.0.22 (src:faea52f3; feat:607245837, client:Agave)
```

```shell
$ anchor --version   
anchor-cli 0.30.1
```

```shell
$ node --version
v22.13.1
```

```shell
$ npm --version
11.0.0
```

```shell
$ yarn --version
1.22.22
```

* 编译

```shell
$ anchor build --arch sbf
```

* 运行单元测试

```shell
$ yarn install
$ anchor test --arch sbf
$ cargo test-sbf
```

* 启动 solana 本地测试节点

```shell
$ solana-test-validator
```

* 部署

```shell
$ anchor deploy
```

* 验证 IDL

```shell
$ anchor idl init --filepath target/idl/batch_transfer.json
```
