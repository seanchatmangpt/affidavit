import * as fsS from "node:fs";
import * as fs from "node:fs/promises";
import * as os from "node:os";
// From my limited testing on Windows, `fs.readlink` returns a full path to 'C:\whatever'. So
// flipping the slashes and prefixing the fake root path is enough to get my sample pnpm project
// working correctly.
const normalizeWindowsLink = function () {
    if (os.platform() === "win32") {
        return path => `/${path.replaceAll("\\", "/")}`;
    }
    else {
        return path => path;
    }
}();
export const defaultAsyncFileSystem = {
    directoryExists: async (path) => {
        try {
            const stat = await fs.stat(path);
            return stat.isDirectory();
        }
        catch {
            return false;
        }
    },
    fileExists: async (path) => {
        try {
            const stat = await fs.stat(path);
            return stat.isFile();
        }
        catch {
            return false;
        }
    },
    readFileJSON: async (path) => JSON.parse(await fs.readFile(path, "utf8")),
    readLink: async (path) => {
        try {
            return normalizeWindowsLink(await fs.readlink(path));
        }
        catch {
            return undefined;
        }
    },
};
export const defaultSyncFileSystem = {
    directoryExists: path => {
        try {
            const stat = fsS.statSync(path);
            return stat.isDirectory();
        }
        catch {
            return false;
        }
    },
    fileExists: path => {
        try {
            const stat = fsS.statSync(path);
            return stat.isFile();
        }
        catch {
            return false;
        }
    },
    readFileJSON: (path) => JSON.parse(fsS.readFileSync(path, "utf8")),
    readLink: path => {
        try {
            return normalizeWindowsLink(fsS.readlinkSync(path));
        }
        catch {
            return undefined;
        }
    },
};
//# sourceMappingURL=fs.js.map