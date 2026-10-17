import ts from "typescript";
import { CompilerHostWrapper } from "./multiCompilerHost.js";
import { allBuildTools, getResolutionKinds } from "../utils.js";
const extensions = new Set([".jsx", ".tsx", ".js", ".ts", ".mjs", ".cjs", ".mts", ".cjs"]);
function getEntrypoints(fs, exportsObject, options) {
    var _a;
    var _b;
    if (options === null || options === void 0 ? void 0 : options.entrypoints) {
        return options.entrypoints.map((e) => formatEntrypointString(e, fs.packageName));
    }
    if (exportsObject === undefined && fs) {
        const rootDir = `/node_modules/${fs.packageName}`;
        const proxies = getProxyDirectories(rootDir, fs);
        if (proxies.length === 0) {
            if (options === null || options === void 0 ? void 0 : options.entrypointsLegacy) {
                return fs
                    .listFiles()
                    .filter((f) => !ts.isDeclarationFileName(f) && extensions.has(f.slice(f.lastIndexOf("."))))
                    .map((f) => "." + f.slice(rootDir.length));
            }
            return ["."];
        }
        return proxies;
    }
    const detectedSubpaths = getSubpaths(exportsObject);
    if (detectedSubpaths.length === 0 && hasExportTarget(exportsObject)) {
        detectedSubpaths.push(".");
    }
    const included = unique([
        ...detectedSubpaths,
        ...((_b = (_a = options === null || options === void 0 ? void 0 : options.includeEntrypoints) === null || _a === void 0 ? void 0 : _a.map((e) => formatEntrypointString(e, fs.packageName))) !== null && _b !== void 0 ? _b : []),
    ]);
    if (!(options === null || options === void 0 ? void 0 : options.excludeEntrypoints)) {
        return included;
    }
    return included.filter((entrypoint) => {
        return !options.excludeEntrypoints.some((exclusion) => {
            if (typeof exclusion === "string") {
                return formatEntrypointString(exclusion, fs.packageName) === entrypoint;
            }
            return exclusion.test(entrypoint);
        });
    });
}
function formatEntrypointString(path, packageName) {
    return (path === "." || path.startsWith("./")
        ? path
        : path === packageName
            ? "."
            : path.startsWith(`${packageName}/`)
                ? `.${path.slice(packageName.length)}`
                : `./${path}`).trim();
}
function getSubpaths(exportsObject) {
    var _a;
    if (!exportsObject || typeof exportsObject !== "object" || Array.isArray(exportsObject)) {
        return [];
    }
    const keys = Object.keys(exportsObject);
    if ((_a = keys[0]) === null || _a === void 0 ? void 0 : _a.startsWith(".")) {
        return keys.filter((key) => hasExportTarget(exportsObject[key]));
    }
    return keys.flatMap((key) => getSubpaths(exportsObject[key]));
}
function hasExportTarget(exportsObject) {
    if (exportsObject === null || exportsObject === undefined) {
        return false;
    }
    if (typeof exportsObject !== "object") {
        return true;
    }
    if (Array.isArray(exportsObject)) {
        return exportsObject.some(hasExportTarget);
    }
    return Object.keys(exportsObject).some((key) => hasExportTarget(exportsObject[key]));
}
function getProxyDirectories(rootDir, fs) {
    const vendorDirectories = new Set();
    const proxyDirectories = [];
    const files = fs.listFiles().sort((a, b) => a.length - b.length);
    for (const file of files) {
        if (file.startsWith(rootDir) && file.endsWith("/package.json")) {
            try {
                const packageJson = JSON.parse(fs.readFile(file));
                if (packageJson.name && !packageJson.name.startsWith(fs.packageName)) {
                    // Name unrelated to the root package, this is a vendored package
                    const vendorDir = file.slice(0, file.lastIndexOf("/"));
                    vendorDirectories.add(vendorDir);
                }
                else if ("main" in packageJson && !isInsideVendorDirectory(file)) {
                    // No name or name starting with root package name, this is intended to be an entrypoint
                    const proxyDir = "." + file.slice(rootDir.length, file.lastIndexOf("/"));
                    proxyDirectories.push(proxyDir);
                }
            }
            catch { }
        }
    }
    return proxyDirectories.sort((a, b) => {
        return ts.comparePathsCaseInsensitive(a, b);
    });
    function isInsideVendorDirectory(file) {
        return !!ts.forEachAncestorDirectory(file, (dir) => {
            if (vendorDirectories.has(dir)) {
                return true;
            }
        });
    }
}
export function getEntrypointInfo(packageName, fs, hosts, options) {
    const packageJson = JSON.parse(fs.readFile(`/node_modules/${packageName}/package.json`));
    let entrypoints = getEntrypoints(fs, packageJson.exports, options);
    if (fs.typesPackage) {
        const typesPackageJson = JSON.parse(fs.readFile(`/node_modules/${fs.typesPackage.packageName}/package.json`));
        const typesEntrypoints = getEntrypoints(fs, typesPackageJson.exports, options);
        entrypoints = unique([...entrypoints, ...typesEntrypoints]);
    }
    const result = {};
    for (const entrypoint of entrypoints) {
        const resolutions = {
            node10: getEntrypointResolution(packageName, hosts.node10, "node10", entrypoint),
            "node16-cjs": getEntrypointResolution(packageName, hosts.node16, "node16-cjs", entrypoint),
            "node16-esm": getEntrypointResolution(packageName, hosts.node16, "node16-esm", entrypoint),
            bundler: getEntrypointResolution(packageName, hosts.bundler, "bundler", entrypoint),
        };
        result[entrypoint] = {
            subpath: entrypoint,
            resolutions,
            hasTypes: Object.values(resolutions).some((r) => { var _a; return (_a = r.resolution) === null || _a === void 0 ? void 0 : _a.isTypeScript; }),
            isWildcard: !!resolutions.bundler.isWildcard,
        };
    }
    return result;
}
function getEntrypointResolution(packageName, host, resolutionKind, entrypoint) {
    if (entrypoint.includes("*")) {
        return { name: entrypoint, resolutionKind, isWildcard: true };
    }
    const moduleSpecifier = packageName + entrypoint.substring(1); // remove leading . before slash
    const importingFileName = resolutionKind === "node16-esm" ? "/index.mts" : "/index.ts";
    const resolutionMode = resolutionKind === "node16-esm"
        ? ts.ModuleKind.ESNext
        : resolutionKind === "node16-cjs"
            ? ts.ModuleKind.CommonJS
            : undefined;
    const resolution = tryResolve();
    const implementationResolution = tryResolve(/*noDtsResolution*/ true);
    const files = resolution
        ? host
            .createPrimaryProgram(resolution.fileName)
            .getSourceFiles()
            .map((f) => f.fileName)
        : undefined;
    return {
        name: entrypoint,
        resolutionKind,
        resolution,
        implementationResolution,
        files,
    };
    function tryResolve(noDtsResolution) {
        var _a;
        const { resolution, trace } = host.resolveModuleName(moduleSpecifier, importingFileName, resolutionMode, noDtsResolution);
        const fileName = (_a = resolution.resolvedModule) === null || _a === void 0 ? void 0 : _a.resolvedFileName;
        if (!fileName) {
            return undefined;
        }
        return {
            fileName,
            isJson: resolution.resolvedModule.extension === ts.Extension.Json,
            isTypeScript: ts.hasTSFileExtension(resolution.resolvedModule.resolvedFileName),
            trace,
        };
    }
}
function unique(array) {
    return array.filter((value, index) => array.indexOf(value) === index);
}
export function getBuildTools(packageJson) {
    if (!packageJson.devDependencies) {
        return {};
    }
    const result = {};
    for (const buildTool of allBuildTools) {
        if (buildTool in packageJson.devDependencies) {
            result[buildTool] = packageJson.devDependencies[buildTool];
        }
    }
    return result;
}
export function getModuleKinds(entrypoints, resolutionOption, hosts) {
    var _a;
    const host = hosts[resolutionOption];
    const result = {};
    for (const resolutionKind of getResolutionKinds(resolutionOption)) {
        for (const entrypoint of Object.values(entrypoints)) {
            const resolution = entrypoint.resolutions[resolutionKind];
            for (const fileName of (_a = resolution.files) !== null && _a !== void 0 ? _a : []) {
                if (!result[fileName]) {
                    result[fileName] = host.getModuleKindForFile(fileName);
                }
            }
            if (resolution.implementationResolution) {
                const fileName = resolution.implementationResolution.fileName;
                if (!result[fileName]) {
                    result[fileName] = host.getModuleKindForFile(fileName);
                }
            }
        }
    }
    return result;
}
//# sourceMappingURL=getEntrypointInfo.js.map