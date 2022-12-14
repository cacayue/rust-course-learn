## 包管理工具

‍

## 常用命令

* cargo run
* cargo build

  *  --release
* cargo check

  * 检查编译

‍

## 版本管理

* Cargo.toml 项目数据描述 -> .Net里的csproj
* Cargo.lock 自动生成的项目依赖清单

  * 可运行项目时需要一起打包
  * 依赖库时加入.gitignore

### 配置

* package配置

  ```Toml
  [package] # 项目信息如下
  name = "world_hello" # 项目名称
  version = "0.1.0" # 当前项目版本
  edition = "2021" # rust大版本
  ```

* 定义项目依赖

  ```Toml
  [dependencies]
  rand = "0.3"
  hammer = { version = "0.5.0"} # 基于官方仓库
  color = { git = "https://github.com/bjz/color-rs" } # 基于git
  geometry = { path = "crates/geometry" } # 基于本地路径
  ```

‍
