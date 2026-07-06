# ChargeGuard / 充电守护

[English](README.md) | 简体中文

ChargeGuard（充电守护）通过 GNOME Quick Settings 管理 Linux 笔记本的原生电池充电模式，并由一个小型特权 D-Bus 后端负责安全写入。

![ChargeGuard in GNOME Quick Settings](docs/assets/chargeguard-gnome-quick-settings.png)

## 它能做什么

ChargeGuard 面向通过 Linux `power_supply` 接口暴露固件充电模式的笔记本电脑。在受支持的机器上，它可以直接从 GNOME Quick Settings 切换标准充电、快速充电、电池养护 / 长寿命等模式。

当前实现关注这些原生充电模式属性：

```text
/sys/class/power_supply/<battery>/charge_types
/sys/class/power_supply/<battery>/charge_type
```

ChargeGuard 只会写入当前设备 `charge_types` 明确报告的模式 token。它不会写入通用百分比阈值，也不会写入旧式厂商私有控制项。

GNOME 扩展会直接显示固件报告的原生模式名，例如 `Fast`、`Standard` 或 `Long_Life`。这些模式 token 不会被 UI 翻译或重新解释。

## 组件

- `chargeguard`：CLI，用于列出电池、查看模式、切换模式和运行诊断。
- `chargeguardd`：特权 system D-Bus 服务，负责 polkit 校验和 sysfs 写入。
- `gnome-shell-extension-chargeguard`：GNOME Quick Settings 扩展，通过 system D-Bus 调用后端。

## 安装

计划中的 Arch/AUR package base 会拆分为两个包：

- `chargeguard`
- `gnome-shell-extension-chargeguard`

AUR 发布后，桌面用户应安装扩展包；它会依赖后端包：

```bash
yay -S gnome-shell-extension-chargeguard
```

只需要 CLI 的用户可以安装后端包：

```bash
yay -S chargeguard
```

AUR 发布前，维护者可以从当前 checkout 构建 Arch 包。详见 [packaging/arch/README.md](packaging/arch/README.md)。

## 启用 GNOME 扩展

首个扩展目标是 GNOME Shell 50。

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

ChargeGuard 当前聚焦具备原生 `charge_type` / `charge_types` 固件模式的笔记本。它不管理 `charge_control_end_threshold` 这类百分比阈值。

详见 [docs/compatibility.md](docs/compatibility.md)。

## Release 和 AUR 维护

维护者发布流程、GitHub Release artifact 和 AUR 发布步骤见 [docs/release.md](docs/release.md)。
