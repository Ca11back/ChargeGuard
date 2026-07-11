import Adw from 'gi://Adw';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

import {ExtensionPreferences, gettext as _} from 'resource:///org/gnome/Shell/Extensions/js/extensions/prefs.js';

import {classifyError} from './protocol.js';

const SERVICE_NAME = 'io.github.ChargeGuard1';
const OBJECT_PATH = '/io/github/ChargeGuard1';
const INTERFACE_NAME = 'io.github.ChargeGuard1.Manager';

const MANAGER_XML = `
<node>
  <interface name="${INTERFACE_NAME}">
    <method name="ListDevices">
      <arg name="devices" type="aa{sv}" direction="out"/>
    </method>
  </interface>
</node>`;

const ManagerProxy = Gio.DBusProxy.makeProxyWrapper(MANAGER_XML);

function unpackVariant(value) {
    while (value instanceof GLib.Variant)
        value = value.deepUnpack();

    return value;
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

function selectDevice(devices) {
    devices = Array.isArray(devices) ? devices : [];

    const presentDevices = devices.filter(device => device.present !== false);
    const supportedDevices = presentDevices.filter(device => device.nativeMode?.supported);

    if (supportedDevices.length === 1)
        return {kind: 'ready', device: supportedDevices[0]};

    if (supportedDevices.length > 1) {
        return {
            kind: 'multiple-supported',
            devices: supportedDevices,
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

function currentModeLabel(device) {
    const nativeMode = device?.nativeMode;
    return nativeMode?.currentLabel || nativeMode?.current || _('Unknown');
}

function deviceDisplayName(device) {
    return device?.modelName || device?.id || _('Battery');
}

function valueOrUnknown(value) {
    if (typeof value === 'string' && value.trim() !== '')
        return value;

    return _('Unknown');
}

function capacityDisplay(device) {
    const capacity = device?.capacity;
    if (typeof capacity === 'number' && Number.isFinite(capacity))
        return `${capacity}%`;

    return _('Unknown');
}

function optionalNumberDisplay(value) {
    if (typeof value === 'number' && Number.isFinite(value))
        return `${value}`;

    return _('Not available');
}

function powerDisplay(device) {
    const microwatts = device?.power?.microwatts;
    if (typeof microwatts !== 'number' || !Number.isFinite(microwatts) || microwatts < 0)
        return null;

    const roundedTenths = Math.round(microwatts / 100000);
    if (roundedTenths === 0)
        return '0 W';

    if (roundedTenths < 100 && roundedTenths % 10 !== 0)
        return `${Math.floor(roundedTenths / 10)}.${roundedTenths % 10} W`;

    return `${Math.round(roundedTenths / 10)} W`;
}

function isNonNegativeNumber(value) {
    return typeof value === 'number' && Number.isFinite(value) && value >= 0;
}

function formatMicroUnits(value, unit) {
    if (!isNonNegativeNumber(value))
        return null;

    const roundedHundredths = Math.round(value / 10000);
    const whole = Math.floor(roundedHundredths / 100);
    const fractional = roundedHundredths % 100;

    if (fractional === 0)
        return `${whole} ${unit}`;

    if (fractional % 10 === 0)
        return `${whole}.${Math.floor(fractional / 10)} ${unit}`;

    return `${whole}.${String(fractional).padStart(2, '0')} ${unit}`;
}

function hasEnergyCapacity(health) {
    return isNonNegativeNumber(health?.fullEnergyMicrowattHours) ||
        isNonNegativeNumber(health?.designEnergyMicrowattHours);
}

function fullCapacityDisplay(health) {
    if (hasEnergyCapacity(health))
        return formatMicroUnits(health?.fullEnergyMicrowattHours, 'Wh');

    return formatMicroUnits(health?.fullChargeMicroampHours, 'Ah');
}

function designCapacityDisplay(health) {
    if (hasEnergyCapacity(health))
        return formatMicroUnits(health?.designEnergyMicrowattHours, 'Wh');

    return formatMicroUnits(health?.designChargeMicroampHours, 'Ah');
}

function healthPercentDisplay(health) {
    const energyFull = health?.fullEnergyMicrowattHours;
    const energyDesign = health?.designEnergyMicrowattHours;
    if (hasEnergyCapacity(health)) {
        if (isNonNegativeNumber(energyFull) && isNonNegativeNumber(energyDesign) && energyDesign > 0)
            return `${(energyFull * 100 / energyDesign).toFixed(1)}%`;

        return null;
    }

    const chargeFull = health?.fullChargeMicroampHours;
    const chargeDesign = health?.designChargeMicroampHours;
    if (isNonNegativeNumber(chargeFull) && isNonNegativeNumber(chargeDesign) && chargeDesign > 0)
        return `${(chargeFull * 100 / chargeDesign).toFixed(1)}%`;

    return null;
}

function nativeModeSourceDisplay(device) {
    const sourcePath = device?.nativeMode?.sourcePath;
    if (typeof sourcePath === 'string' && sourcePath.trim() !== '')
        return sourcePath;

    const source = device?.nativeMode?.source;
    if (typeof source !== 'string' || source.trim() === '')
        return null;

    if (source.startsWith('/'))
        return source;

    const nativePath = device?.nativePath;
    if (typeof nativePath === 'string' && nativePath.trim() !== '')
        return `${nativePath.replace(/\/$/, '')}/${source}`;

    return source;
}

function supportedRawModesDisplay(device) {
    const modes = device?.nativeMode?.supportedModes ?? [];
    const rawModes = modes
        .map(mode => mode?.raw)
        .filter(raw => typeof raw === 'string' && raw.length > 0);

    return rawModes.length > 0 ? rawModes.join(', ') : _('None reported');
}

function addValueRow(group, title, value) {
    group.add(new Adw.ActionRow({
        title,
        subtitle: value,
    }));
}

const ChargeGuardDetailsPage = GObject.registerClass(
class ChargeGuardDetailsPage extends Adw.PreferencesPage {
    _init() {
        super._init({
            title: _('ChargeGuard'),
            icon_name: 'battery-good-symbolic',
        });

        this._client = new ChargeGuardClient();
        this._groups = [];
        this._requestSerial = 0;

        this._renderMessage(_('Loading'), _('Reading battery details'));
    }

    load() {
        const serial = ++this._requestSerial;
        this._renderMessage(_('Loading'), _('Reading battery details'));

        this._client.listDevices()
            .then(devices => {
                if (serial !== this._requestSerial)
                    return;

                this._renderSelection(selectDevice(devices));
            })
            .catch(error => {
                if (serial !== this._requestSerial)
                    return;

                console.error(`ChargeGuard failed to load devices: ${error?.message ?? error}`);
                this._renderError(classifyError(error));
            });
    }

    destroyClient() {
        this._requestSerial += 1;
        this._client?.destroy();
        this._client = null;
    }

    _replaceGroups(groups) {
        for (const group of this._groups)
            this.remove(group);

        this._groups = groups;

        for (const group of groups)
            this.add(group);
    }

    _renderSelection(selection) {
        switch (selection.kind) {
        case 'ready':
            this._renderReady(selection.device);
            break;
        case 'multiple-supported':
            this._renderMultipleSupported(selection);
            break;
        case 'unsupported':
            this._renderMessage(_('Unsupported'), selection.message);
            break;
        default:
            this._renderMessage(_('Error'), _('Unexpected ChargeGuard error'));
            break;
        }
    }

    _renderReady(device) {
        const detailsGroup = new Adw.PreferencesGroup({
            title: _('Battery details'),
        });
        addValueRow(detailsGroup, _('Selected battery'), valueOrUnknown(device?.id));
        addValueRow(detailsGroup, _('Model'), valueOrUnknown(device?.modelName));
        addValueRow(detailsGroup, _('Manufacturer'), valueOrUnknown(device?.manufacturer));
        addValueRow(detailsGroup, _('Status'), valueOrUnknown(device?.status));
        addValueRow(detailsGroup, _('Capacity'), capacityDisplay(device));
        addValueRow(detailsGroup, _('Battery-side power'), powerDisplay(device) ?? _('Not available'));
        addValueRow(detailsGroup, _('Cycle count'), optionalNumberDisplay(device?.health?.cycleCount));
        addValueRow(detailsGroup, _('Capacity level'), valueOrUnknown(device?.health?.capacityLevel));
        addValueRow(detailsGroup, _('Full capacity'), fullCapacityDisplay(device?.health) ?? _('Not available'));
        addValueRow(detailsGroup, _('Design capacity'), designCapacityDisplay(device?.health) ?? _('Not available'));
        addValueRow(detailsGroup, _('Battery health'), healthPercentDisplay(device?.health) ?? _('Not available'));

        const modeGroup = new Adw.PreferencesGroup({
            title: _('Native mode'),
        });
        addValueRow(modeGroup, _('Current mode'), currentModeLabel(device));
        addValueRow(modeGroup, _('Current raw mode token'), valueOrUnknown(device?.nativeMode?.current));
        addValueRow(modeGroup, _('Supported raw mode tokens'), supportedRawModesDisplay(device));
        addValueRow(modeGroup, _('Native mode source'), nativeModeSourceDisplay(device) ?? _('Not available'));

        const diagnosticsGroup = new Adw.PreferencesGroup({
            title: _('Diagnostics'),
        });
        addValueRow(diagnosticsGroup, _('Native sysfs path'), valueOrUnknown(device?.nativePath));

        this._replaceGroups([detailsGroup, modeGroup, diagnosticsGroup]);
    }

    _renderMultipleSupported(selection) {
        const statusGroup = new Adw.PreferencesGroup();
        addValueRow(statusGroup, _('Multiple supported batteries'), selection.message);

        const devicesGroup = new Adw.PreferencesGroup({
            title: _('Supported batteries'),
        });
        for (const device of selection.devices)
            addValueRow(devicesGroup, valueOrUnknown(device?.id), deviceDisplayName(device));

        this._replaceGroups([statusGroup, devicesGroup]);
    }

    _renderError(state) {
        switch (state.kind) {
        case 'permission-denied':
            this._renderMessage(_('Permission denied'), state.message);
            break;
        case 'backend-unavailable':
            this._renderMessage(_('Backend unavailable'), state.message);
            break;
        case 'unsupported':
            this._renderMessage(_('Unsupported'), state.message);
            break;
        default:
            this._renderMessage(_('Error'), state.message || _('Unexpected ChargeGuard error'));
            break;
        }
    }

    _renderMessage(title, message) {
        const group = new Adw.PreferencesGroup();
        addValueRow(group, title, message);
        this._replaceGroups([group]);
    }
});

export default class ChargeGuardPreferences extends ExtensionPreferences {
    fillPreferencesWindow(window) {
        window.set_default_size(520, 640);

        const page = new ChargeGuardDetailsPage();
        window.add(page);
        window.connect('close-request', () => {
            page.destroyClient();
            return false;
        });

        page.load();
    }
}
