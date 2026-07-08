# ChargeGuard / 充电守护

[English](README.md) | 简体中文

ChargeGuard（充电守护）把笔记本固件自带的原生充电模式放进 GNOME Quick
Settings。它面向已经通过 Linux `power_supply` 暴露 `Fast`、`Standard`、
`Long_Life` 这类模式的机器。

Quick Settings tile 会显示当前充电模式，以及电池上报的当前电池侧功率。展开
tile 后可以切换固件报告的模式，也可以打开电池详情页。

![ChargeGuard Quick Settings tile and menu](docs/assets/chargeguard-gnome-quick-settings.png)

## GNOME 体验

ChargeGuard 的位置是补充 GNOME 自带的电池图标和电源模式控制，而不是重复它们。

- 顶层 Quick Settings tile 显示当前原生充电模式，例如 `Long Life`，并在可用时显示当前电池侧瓦数。
- 展开菜单会标识工具归属为 ChargeGuard，显示当前模式/功率摘要，列出可写入的固件模式，并提供 `Details...` 入口。
- 实际可选模式保留固件报告的原始 token，例如 `Fast`、`Standard`、`Long_Life`。

## 电池详情

详情页用于承载不适合长期占用 Quick Settings 顶层的信息：电池身份、健康信息、当前功率，以及 ChargeGuard 正在使用的 Linux 接口。

![ChargeGuard battery details page](docs/assets/chargeguard-battery-details.png)

如果机器通过 `power_supply` 上报了对应数据，详情页可以显示：

- 当前选中的电池、型号、制造商、状态和电量
- 来自 `power_now` 的电池侧功率
- 循环次数和容量等级
- 满充容量、设计容量和计算出的电池健康度
- 当前原生模式、当前原始模式 token、支持的原始 token，以及完整的原生模式来源路径

这些字段取决于硬件和内核暴露的 sysfs 属性。缺少可选属性时，ChargeGuard 会显示为不可用，而不是把它当作错误。

## 范围边界

ChargeGuard 是固件原生充电模式的前端。它不会自己发明一套软件充电策略，不替代 GNOME 的电源模式控制，也不是通用的笔记本电源调优工具。

当前支持的原生模式接口是：

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

模式切换会先根据当前电池 `charge_types` 报告的 allow-list 校验。ChargeGuard 不写入
`charge_control_end_threshold` 这类通用百分比阈值，也不调整 CPU、磁盘、USB、Wi-Fi、platform profile、键盘背光或运行时省电策略。

需要通用性能/省电策略时，应该使用 TLP、GNOME 电源模式或其他电源管理工具。你的笔记本已经有固件原生充电模式，并且你想把它清晰地接入 GNOME 和 CLI 诊断时，才使用 ChargeGuard。

## 组件

- `chargeguard`：CLI，用于列出电池、查看模式、切换模式和运行诊断。
- `chargeguardd`：特权 system D-Bus 服务，负责 polkit 校验和 sysfs 写入。
- `gnome-shell-extension-chargeguard`：GNOME Quick Settings 扩展，通过 system D-Bus 调用后端。

## 安装

Arch 桌面用户应安装 GNOME 扩展包；它会依赖后端包：

```bash
yay -S gnome-shell-extension-chargeguard
```

只需要 CLI 的用户可以安装后端包：

```bash
yay -S chargeguard
```

维护者可以从当前 checkout 构建 Arch 包。详见
[packaging/arch/README.md](packaging/arch/README.md)。

## 启用 GNOME 扩展

扩展目标是 GNOME Shell 50。

安装包后，根据你的会话类型重新加载或重启 GNOME Shell，然后启用扩展：

```bash
gnome-extensions enable chargeguard@github.com
gnome-extensions info chargeguard@github.com
```

CLI 或扩展查询 ChargeGuard 时，后端会通过 system D-Bus 自动激活。

## CLI

```bash
chargeguard list
chargeguard current
chargeguard modes
chargeguard doctor --json
chargeguard raw BAT1
```

只有在确认本机报告的受支持模式后，才执行 `set`：

```bash
chargeguard set Long_Life --device BAT1
```

## 安全模型

- 读取操作不需要 polkit 授权。
- 模式切换会经过 `chargeguardd` 和 polkit。
- 后端只写入目标电池当前 `charge_types` 文件报告的原生模式 token。
- 没有可枚举可写原生模式列表的设备会被报告为不支持或只读。

## 兼容性

ChargeGuard 当前聚焦具备原生 `charge_type` / `charge_types` 固件模式的笔记本。健康和功率详情取决于每块电池在 `power_supply` 下实际报告的可选属性。

详见 [docs/compatibility.md](docs/compatibility.md)。

## Release 和 AUR 维护

维护者发布流程、GitHub Release artifact 和 AUR 发布步骤见 [docs/release.md](docs/release.md)。
