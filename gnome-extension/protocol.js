import Gio from 'gi://Gio';

import {classifyRemoteError} from './error-contract.js';

export {classifyRemoteError} from './error-contract.js';

export function classifyError(error) {
    let remoteName = null;
    try {
        remoteName = Gio.DBusError.get_remote_error(error);
    } catch {
        // Local/protocol errors have no remote identity and remain generic.
    }

    return classifyRemoteError(remoteName, error?.message ?? String(error));
}
