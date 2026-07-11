import {classifyRemoteError} from '../gnome-extension/error-contract.js';

function assertEqual(actual, expected, context) {
    if (actual !== expected)
        throw new Error(`${context}: expected ${expected}, got ${actual}`);
}

const cases = [
    ['io.github.ChargeGuard1.Error.PermissionDenied', 'permission-denied'],
    ['io.github.ChargeGuard1.Error.ServiceUnavailable', 'backend-unavailable'],
    ['org.freedesktop.DBus.Error.ServiceUnknown', 'backend-unavailable'],
    ['org.freedesktop.DBus.Error.NameHasNoOwner', 'backend-unavailable'],
    ['io.github.ChargeGuard1.Error.UnsupportedDevice', 'unsupported'],
    ['io.github.ChargeGuard1.Error.NoNativeModeInterface', 'unsupported'],
    ['io.github.ChargeGuard1.Error.NativeModesNotEnumerable', 'unsupported'],
    ['io.github.ChargeGuard1.Error.InvalidMode', 'error'],
    ['io.github.ChargeGuard1.Error.SysfsReadFailed', 'error'],
    ['io.github.ChargeGuard1.Error.SysfsWriteFailed', 'error'],
    ['io.github.ChargeGuard1.Error.ReadbackMismatch', 'error'],
    ['io.github.ChargeGuard1.Error.InternalError', 'error'],
    ['io.github.ChargeGuard1.Error.FutureError', 'error'],
];

for (const [remoteName, kind] of cases)
    assertEqual(classifyRemoteError(remoteName, 'diagnostic').kind, kind, remoteName);

assertEqual(classifyRemoteError(null, 'permission denied').kind, 'error', 'human permission text');
assertEqual(classifyRemoteError(null, 'service is not activatable').kind, 'error', 'human service text');
assertEqual(classifyRemoteError(null, 'not supported').kind, 'error', 'human support text');
