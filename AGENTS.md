# AGENTS.md

## 1. 项目基本信息

* **项目名 / Repo name:** `agent-env-guard`
* **CLI 名:** `maskrun`
* **主要语言:** Rust
* **跨平台**: 支持全部平台, linux, windows, mac
* **项目类型:** CLI 工具
* **核心目标:** 在执行子命令时，对 stdout / stderr 中出现的环境变量密钥值进行脱敏，减少 secrets 泄露到终端、日志、agent 记录、任务 transcript 中的风险。

示例：

```bash
# 正常使用, 让CLI可以使用环境变量作中的凭据
maskrun -- curl "https://api.example.com?key=${APIKEY}"
# output: [正常执行结果]

# 非正常使用, 不会输出明文执行结果, 而是输出mask内容
maskrun -- echo "key=$APIKEY"
## output: key=s**********6
```


---

## 2. 项目定位

本项目适合：

* 本地开发者运行带有敏感环境变量的命令。
* AI coding agents 执行 shell 命令时过滤输出。
* OpenClaw 等 agent CLI / agent runtime 场景。
* 需要保留命令输出，但不希望密钥进入日志的工作流。

本项目解决的问题：

* agent 执行命令时直接打印 `$APIKEY`、`$TOKEN`、`$SECRET` 等敏感值。

本项目不解决的问题：

* 不管理任何凭据
* 不阻止子进程读取环境变量。
* 不阻止子进程把密钥发送到网络。
* 不阻止子进程把密钥写入文件。
* 不提供 sandbox、容器隔离、权限隔离能力。
* 不替代 secret manager、IAM、CI secret masking 或系统级安全策略。

文档中不要把本项目描述为完整的安全沙箱。

---

## 3. 核心设计原则

### 3.1 明确安全边界

`maskrun` 只做输出脱敏。

它不是：

* sandbox
* jail
* container
* secret manager
* network firewall
* syscall filter

推荐描述：

```text
maskrun filters command output to reduce accidental secret exposure.
```

避免描述：

```text
maskrun securely runs untrusted commands.
```

---

### 3.2 行为可预测

优先使用确定性规则：

* 读取环境变量。
* 找出疑似 secret 的变量值。
* 对 stdout / stderr 做精确字符串替换。
* 保留原命令退出码。

永远不做复杂智能识别。
值替换更容易测试和解释。

---

### 3.3 参数 as 子命令, 完全透明

`maskrun` 应尽量保持透明包装：

* 不修改 command args。
* 不合并 stdout / stderr
* 不吞掉退出码。
* 不默认静默失败。
* 不吞掉 child process 获取当前进程的env

---

## 4. 推荐 CLI 设计

### 4.1 基本命令格式

```bash

maskrun [options] -- <raw_command> [raw_args...]
```

示例：

```bash
maskrun -- echo "$APIKEY"
maskrun -- bash -lc 'echo "$APIKEY"'
maskrun -- curl "https://api.example.com?key=${APIKEY}"
```

说明：

* `--` 后面的所有内容都属于子命令。
* `maskrun` 不应该尝试解析子命令参数。

---

## 5. 配置文件
标准配置文件位置的toml配置文件

结构:
```toml
[filter]
exact = [
  "API_KEY",
  "SECRET",
  "PASSWORD",
]

glob = [
  "*_KEY",
  "*_TOKEN",
  "*_SECRET",
]

regex = [
  "(?i)^.*password.*$",
]
```

## 6. 实现原理
- 只针对输出进行过滤替换
- 步骤
  1. 读取所有环境变量KV, **命中配置**的values作为字符串对比数组
  2. 遍历执行输出, 字符串匹配**1.**中的字符串
  3. 如果命中, 进行mask



