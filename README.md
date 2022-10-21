# iamtxt签到工具
- 使用 `Rust` 编写

## 环境
- Windows 10 x86_64 / Linux x86_64 / macOS arm/x86_64 **可以直接下载最新[发行版](https://github.com/azureqaq/iamtxtAttendance/releases)**
- macOS arm **需要自己编译**

## 编译
- 安装 `Rust`：[rust-lang](https://www.rust-lang.org/zh-CN/tools/install)
- [下载](https://github.com/azureqaq/iamtxtAttendance/archive/refs/heads/master.zip) 或者 `git clone` [本仓库](https://github.com/azureqaq/iamtxtAttendance.git)，切换到 `master` 分支，让后执行 `cargo build --release` 即可

## 功能
- 签到
- 信息查询


## 使用方法
1. 得到二进制可执行文件：`iamtxt.exe` / `iamtxt`
2. 在Windows上打开 `cmd` *可以通过按下win+R键，并输入cmd*，拖入二进制文件来运行
3. 在Unix-like上，直接通过终端运行即可


## 参数说明
- `--init`：创建文件夹及默认文件，如果不执行次操作，功能无法使用
- `--att`：为所有配置文件中开启了的账号签到
- `--clean`：在记录状态的文件中删除不需要的
- `--uninstall`：删除所有本工具在安装时创建的文件/文件夹
- `--info`: 查询积分


## 配置文件
- `enable`：是否开启
- `name`：用户名
- `pwd`：对应的密码
- `retry_times`：重试次数 **正整数**
- **注意**：配置文件位置可以在首次执行 `--init` 时看到具体路径

## 推荐的使用方法
- 在操作系统中配置计划任务即可

