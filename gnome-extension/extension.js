import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

import {Extension, gettext as _} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';

const SERVICE_NAME = 'io.github.ChargeGuard1';
const OBJECT_PATH = '/io/github/ChargeGuard1';
const INTERFACE_NAME = 'io.github.ChargeGuard1.Manager';

const ICONS = {
    ready: 'battery-good-symbolic',
    loading: 'content-loading-symbolic',
    warning: 'dialog-warning-symbolic',
    error: 'dialog-error-symbolic',
};

const MANAGER_XML = `
<node>
  <interface name="${INTERFACE_NAME}">
    <method name="ListDevices">
      <arg name="devices" type="aa{sv}" direction="out"/>
    </method>
    <method name="GetDevice">
      <arg name="id" type="s" direction="in"/>
      <arg name="device" type="a{sv}" direction="out"/>
    </method>
    <method name="SetNativeMode">
      <arg name="id" type="s" direction="in"/>
      <arg name="mode" type="s" direction="in"/>
      <arg name="device" type="a{sv}" direction="out"/>
    </method>
    <method name="GetDiagnostics">
      <arg name="diagnostics" type="a{sv}" direction="out"/>
    </method>
  </interface>
</node>`;

const ManagerProxy = Gio.DBusProxy.makeProxyWrapper(MANAGER_XML);

function unpackVariant(value) {
    while (value instanceof GLib.Variant)
        value = value.deepUnpack();

    return value;
}

function themedIcon(iconName) {
    return Gio.ThemedIcon.new(iconName);
}

function getMapValue(map, key) {
    map = unpackVariant(map);

    if (map instanceof Map)
        return unpackVariant(map.get(key));

    if (map && typeof map === 'object')
        return unpackVariant(map[key]);

    return null;
}

function jsonFromMap(map) {
    const json = getMapValue(map, 'json');

    if (typeof json !== 'string')
        throw new Error('ChargeGuard D-Bus payload is missing the json field');

    return JSON.parse(json);
}

function firstOutArg(result) {
    if (Array.isArray(result) && result.length === 1)
        return result[0];

    return result;
}

function remoteErrorName(error) {
    try {
        return Gio.DBusError.get_remote_error(error);
    } catch {
        return null;
    }
}

function classifyError(error) {
    const remoteName = remoteErrorName(error) ?? '';
    const message = error?.message ?? String(error);
    const normalized = `${remoteName} ${message}`.toLowerCase();

    if (normalized.includes('accessdenied') || normalized.includes('access denied') ||
        normalized.includes('permission'))
        return {kind: 'permission-denied', message};

    if (normalized.includes('serviceunknown') || normalized.includes('service unknown') ||
        normalized.includes('not provided') || normalized.includes('not activatable'))
        return {kind: 'backend-unavailable', message};

    if (normalized.includes('notsupported') || normalized.includes('not supported'))
        return {kind: 'unsupported', message};

    return {kind: 'error', message};
}

class ChargeGuardClient {
    constructor() {
        this._proxy = new ManagerProxy(
            Gio.DBus.system,
            SERVICE_NAME,
            OBJECT_PATH
        );
    }

    listDevices() {
        return this._call('ListDevices')
            .then(devices => firstOutArg(devices).map(jsonFromMap));
    }

    setNativeMode(deviceId, mode) {
        return this._call('SetNativeMode', deviceId, mode)
            .then(device => jsonFromMap(firstOutArg(device)));
    }

    _call(method, ...args) {
        return new Promise((resolve, reject) => {
            this._proxy[`${method}Remote`](...args, (result, error) => {
                if (error) {
                    reject(error);
                    return;
                }

                resolve(result);
            });
        });
    }

    destroy() {
        this._proxy?.run_dispose();
        this._proxy = null;
    }
}

function modeDisplayName(mode) {
    return mode?.label || mode?.raw || _('Unknown');
}

function deviceDisplayName(device) {
    return device?.modelName || device?.id || _('Battery');
}

function currentModeLabel(device) {
    const nativeMode = device?.nativeMode;
    return nativeMode?.currentLabel || nativeMode?.current || _('Unknown');
}

function selectDevice(devices) {
    const presentDevices = devices.filter(device => device.present !== false);
    const supportedDevices = presentDevices.filter(device => device.nativeMode?.supported);

    if (supportedDevices.length === 1)
        return {kind: 'ready', device: supportedDevices[0]};

    if (supportedDevices.length > 1) {
        return {
            kind: 'unsupported',
            message: _('Multiple supported batteries were detected. Device selection is not available yet.'),
        };
    }

    if (presentDevices.length === 1) {
        return {
            kind: 'unsupported',
            message: presentDevices[0].nativeMode?.message ||
                _('This battery does not expose native charge modes.'),
        };
    }

    if (presentDevices.length === 0) {
        return {
            kind: 'unsupported',
            message: _('No present battery was detected.'),
        };
    }

    return {
        kind: 'unsupported',
        message: _('No supported battery charge-mode interface was detected.'),
    };
}

const ChargeGuardToggle = GObject.registerClass(
class ChargeGuardToggle extends QuickSettings.QuickMenuToggle {
    _init(client) {
        super._init({
            title: _('ChargeGuard'),
            subtitle: _('Loading'),
            toggleMode: false,
        });

        this._client = client;
        this._requestSerial = 0;
        this._state = {kind: 'loading'};
        this.gicon = themedIcon(ICONS.loading);

        this.menu.setHeader(ICONS.loading, _('ChargeGuard'), _('Loading battery charge modes'));

        this._statusSection = new PopupMenu.PopupMenuSection();
        this._modeSection = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._statusSection);
        this.menu.addMenuItem(this._modeSection);

        this.connect('destroy', () => {
            this._requestSerial += 1;
            this._client = null;
        });

        this.refresh();
    }

    refresh({showLoading = true} = {}) {
        const serial = ++this._requestSerial;
        if (showLoading)
            this._render({kind: 'loading'});

        this._client.listDevices()
            .then(devices => {
                if (serial !== this._requestSerial)
                    return;

                this._render(selectDevice(devices));
            })
            .catch(error => {
                if (serial !== this._requestSerial)
                    return;

                logError(error, 'ChargeGuard failed to load devices');
                this._render(classifyError(error));
            });
    }

    _setMode(device, mode) {
        const serial = ++this._requestSerial;
        this._render({kind: 'loading', device});

        this._client.setNativeMode(device.id, mode.raw)
            .then(updatedDevice => {
                if (serial !== this._requestSerial)
                    return;

                this._render({kind: 'ready', device: updatedDevice});
            })
            .catch(error => {
                if (serial !== this._requestSerial)
                    return;

                logError(error, 'ChargeGuard failed to set native mode');
                this._render({
                    ...classifyError(error),
                    device,
                });
            });
    }

    _render(state) {
        this._state = state;
        this._statusSection.removeAll();
        this._modeSection.removeAll();

        switch (state.kind) {
        case 'loading':
            this._renderStatus(ICONS.loading, _('Loading'), _('Reading battery charge modes'));
            break;
        case 'ready':
            this._renderReady(state.device);
            break;
        case 'permission-denied':
            this._renderStatus(ICONS.warning, _('Permission denied'), state.message);
            if (state.device)
                this._renderModes(state.device, false);
            break;
        case 'unsupported':
            this._renderStatus(ICONS.warning, _('Unsupported'), state.message);
            break;
        case 'backend-unavailable':
            this._renderStatus(ICONS.error, _('Backend unavailable'), state.message);
            break;
        default:
            this._renderStatus(ICONS.error, _('Error'), state.message || _('Unexpected ChargeGuard error'));
            if (state.device)
                this._renderModes(state.device, false);
            break;
        }
    }

    _renderReady(device) {
        const modeInfo = device.nativeMode;
        const current = currentModeLabel(device);
        const writable = modeInfo?.writable === true;
        const subtitle = writable ? current : `${current} (${_('read-only')})`;

        this.title = _('ChargeGuard');
        this.subtitle = subtitle;
        this.gicon = themedIcon(ICONS.ready);
        this.menu.setHeader(ICONS.ready, deviceDisplayName(device), subtitle);

        if (!writable) {
            const item = new PopupMenu.PopupMenuItem(_('Mode switching is not writable on this device.'));
            item.sensitive = false;
            this._statusSection.addMenuItem(item);
        }

        this._renderModes(device, writable);
    }

    _renderModes(device, writable) {
        const current = device.nativeMode?.current;
        const modes = device.nativeMode?.supportedModes ?? [];

        if (modes.length === 0) {
            const item = new PopupMenu.PopupMenuItem(_('No supported modes were reported.'));
            item.sensitive = false;
            this._modeSection.addMenuItem(item);
            return;
        }

        for (const mode of modes) {
            const item = new PopupMenu.PopupMenuItem(modeDisplayName(mode));
            const isCurrent = mode.raw === current;

            item.setOrnament(isCurrent ? PopupMenu.Ornament.CHECK : PopupMenu.Ornament.NONE);
            item.sensitive = writable && !isCurrent;
            item.connectObject('activate', () => this._setMode(device, mode), this);
            this._modeSection.addMenuItem(item);
        }
    }

    _renderStatus(iconName, title, message) {
        this.title = _('ChargeGuard');
        this.subtitle = title;
        this.gicon = themedIcon(iconName);
        this.menu.setHeader(iconName, _('ChargeGuard'), title);

        if (message) {
            const item = new PopupMenu.PopupMenuItem(message);
            item.sensitive = false;
            this._statusSection.addMenuItem(item);
        }
    }
});

const ChargeGuardIndicator = GObject.registerClass(
class ChargeGuardIndicator extends QuickSettings.SystemIndicator {
    _init(quickSettingsMenu) {
        super._init();

        this._quickSettingsMenu = quickSettingsMenu;
        this._client = new ChargeGuardClient();
        this._toggle = new ChargeGuardToggle(this._client);
        this.quickSettingsItems.push(this._toggle);

        this._menuOpenSignalId = this._quickSettingsMenu.connect(
            'open-state-changed',
            (_menu, isOpen) => {
                if (isOpen)
                    this._toggle.refresh({showLoading: false});
            }
        );
    }

    destroy() {
        if (this._menuOpenSignalId) {
            this._quickSettingsMenu.disconnect(this._menuOpenSignalId);
            this._menuOpenSignalId = null;
        }

        this._quickSettingsMenu = null;
        this.quickSettingsItems.forEach(item => item.destroy());
        this._toggle = null;
        this._client?.destroy();
        this._client = null;
        super.destroy();
    }
});

export default class ChargeGuardExtension extends Extension {
    enable() {
        const quickSettings = Main.panel.statusArea.quickSettings;
        this._indicator = new ChargeGuardIndicator(quickSettings.menu);
        quickSettings.addExternalIndicator(this._indicator);
    }

    disable() {
        this._indicator?.destroy();
        this._indicator = null;
    }
}
