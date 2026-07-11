const ERROR_PREFIX = 'io.github.ChargeGuard1.Error.';

const ERROR_KINDS = new Map([
    [`${ERROR_PREFIX}PermissionDenied`, 'permission-denied'],
    [`${ERROR_PREFIX}ServiceUnavailable`, 'backend-unavailable'],
    ['org.freedesktop.DBus.Error.ServiceUnknown', 'backend-unavailable'],
    ['org.freedesktop.DBus.Error.NameHasNoOwner', 'backend-unavailable'],
    [`${ERROR_PREFIX}UnsupportedDevice`, 'unsupported'],
    [`${ERROR_PREFIX}NoNativeModeInterface`, 'unsupported'],
    [`${ERROR_PREFIX}NativeModesNotEnumerable`, 'unsupported'],
    [`${ERROR_PREFIX}InvalidMode`, 'error'],
    [`${ERROR_PREFIX}SysfsReadFailed`, 'error'],
    [`${ERROR_PREFIX}SysfsWriteFailed`, 'error'],
    [`${ERROR_PREFIX}ReadbackMismatch`, 'error'],
    [`${ERROR_PREFIX}InternalError`, 'error'],
]);

export function classifyRemoteError(remoteName, message) {
    return {
        kind: ERROR_KINDS.get(remoteName) ?? 'error',
        message,
    };
}
