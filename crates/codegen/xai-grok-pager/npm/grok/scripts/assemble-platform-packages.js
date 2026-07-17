#!/usr/bin/env node
// Assemble the six per-platform npm packages prior to `npm publish`.
//
// `--check-assets` validates the complete vendored Warp manifest plus the
// package notices/license inputs without reading binaries or writing anything.
// `--output-root <dir>` stages complete per-platform packages outside the
// source tree (the historical default remains the adjacent npm directory).
//
// For each supported (platform, arch) target this:
//   1. Copies the package template into a staging directory
//   2. Brotli-compresses the built binary into `bin/<bin>.br`
//   3. Stamps the sub-package's version to match the meta package
//   4. Copies product notices and the exact audited Warp themes license bytes
//
// Each per-platform package is its own npm publish target. The meta package
// (`@xai-official/grok`) lists all six as `optionalDependencies` pinned to
// the same version; npm installs only the one matching the host's
// `os` + `cpu` filters.
//
// Why brotli? npm's tarball ceiling is ~200 MB and the raw grok binary is
// 100–150 MB per platform. Brotli at max quality cuts that to 30–40 MB,
// leaves plenty of headroom for binary growth, and is decoded by Node's
// built-in zlib.brotliDecompressSync (no native deps required).

'use strict';

const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const { promisify, TextDecoder } = require('util');
const zlib = require('zlib');

const brotliCompress = promisify(zlib.brotliCompress);
const utf8Decoder = new TextDecoder('utf-8', { fatal: true });

const npmRoot = path.resolve(__dirname, '..', '..');
const repositoryRoot = process.env.XAI_ROOT
    ? path.resolve(process.env.XAI_ROOT)
    : path.resolve(npmRoot, '..', '..', '..', '..');
const pagerRenderRoot = path.resolve(npmRoot, '..', '..', 'xai-grok-pager-render');
const warpThemesRoot = path.join(pagerRenderRoot, 'assets', 'warp-themes');

const TOOL_NOTICES_SOURCE = path.resolve(
    npmRoot, '..', '..', 'xai-grok-tools', 'THIRD_PARTY_NOTICES.md');
const SHELL_NOTICES_SOURCE = path.resolve(
    npmRoot, '..', '..', 'xai-grok-shell', 'THIRD_PARTY_NOTICES.md');
const PAGER_RENDER_NOTICES_SOURCE = path.join(pagerRenderRoot, 'THIRD_PARTY_NOTICES.md');
const NOTICES_SOURCES = Object.freeze([
    Object.freeze({
        source: TOOL_NOTICES_SOURCE,
        bytes: 9063,
        sha256: 'c39f8e8bf05f5a605d2f0bf5c7c0ac3ed41042cc80aba47c25e7138771d7be10',
        requiredMarker: 'src/implementations/grok_build/web_search/',
    }),
    Object.freeze({
        source: SHELL_NOTICES_SOURCE,
        bytes: 1846,
        sha256: '4152f0550bc3b2dc40d0053d384b54f8c57e23124fc9798e250a4b9808b6ba3b',
        requiredMarker: 'src/auth/codex/',
    }),
    Object.freeze({
        source: PAGER_RENDER_NOTICES_SOURCE,
        bytes: 2185,
        sha256: '422b2a9a94343b3cb9dd448e40503dba5de87a5464bafa610e8c27407ee9ad24',
        requiredMarker: 'warpdotdev/themes',
    }),
]);
const NOTICES_NAME = 'THIRD_PARTY_NOTICES.md';
const WARP_THEMES_LICENSE_NAME = 'WARP_THEMES_LICENSE';

const META_PKG_JSON = path.resolve(__dirname, '..', 'package.json');
const EXPECTED_WARP_SOURCE = 'https://github.com/warpdotdev/themes.git';
const EXPECTED_WARP_REVISION = 'b385044250f1ed3c9379ab34a8fe82f02fdffaa4';
const EXPECTED_WARP_LICENSE_SHA256 =
    'c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4';
const EXPECTED_VENDOR_MANIFEST_SHA256 =
    'dc1b04a7ea2639d0e78f6810c433f12940304c921d2b7226178a9deed871f0cc';
const EXPECTED_THEME_COUNT = 340;
const EXPECTED_CATEGORY_COUNTS = Object.freeze({
    base16: 178,
    standard: 134,
    special_edition: 8,
    stradicat: 1,
    warp_bundled: 19,
});
const CATEGORIES = Object.freeze(Object.keys(EXPECTED_CATEGORY_COUNTS));
const ROOT_FILES = Object.freeze([
    'LICENSE',
    'README.md',
    'UPSTREAM_REVISION',
    'VENDOR_MANIFEST.json',
]);
const MAX_THEME_BYTES = 1024 * 1024;
const MAX_LICENSE_BYTES = 256 * 1024;
const MAX_MANIFEST_BYTES = 4 * 1024 * 1024;
const SHA256_RE = /^[0-9a-f]{64}$/;
const REVISION_RE = /^[0-9a-f]{40}$/;
const PORTABLE_COMPONENT_RE = /^[A-Za-z0-9._-]+$/;
const WINDOWS_RESERVED_CHARACTER_RE = /[<>:"|?*]/;
const BIDI_CONTROL_RE = /[\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u;
const WINDOWS_RESERVED_NAMES = new Set([
    'CON', 'PRN', 'AUX', 'NUL',
    ...Array.from({ length: 9 }, (_, index) => `COM${index + 1}`),
    ...Array.from({ length: 9 }, (_, index) => `LPT${index + 1}`),
]);
const MAX_PORTABLE_COMPONENT_BYTES = 255;
const MAX_PORTABLE_PATH_BYTES = 4096;

const TARGETS = Object.freeze([
    {
        platform: 'darwin', arch: 'arm64', binName: 'grok',
        envVar: 'GROK_DARWIN_ARM64',
        defaultSource: path.join(repositoryRoot, 'target', 'release', 'xai-grok-pager'),
    },
    {
        platform: 'darwin', arch: 'x64', binName: 'grok',
        envVar: 'GROK_DARWIN_X64',
        defaultSource: path.join(
            repositoryRoot, 'target', 'x86_64-apple-darwin', 'release', 'xai-grok-pager'),
    },
    {
        platform: 'linux', arch: 'x64', binName: 'grok',
        envVar: 'GROK_LINUX_X64',
        defaultSource: path.join(
            repositoryRoot, 'target', 'explorer_cross_x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-gnu', 'release', 'xai-grok-pager'),
    },
    {
        platform: 'linux', arch: 'arm64', binName: 'grok',
        envVar: 'GROK_LINUX_ARM64',
        defaultSource: path.join(
            repositoryRoot, 'target', 'explorer_cross_aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-gnu', 'release', 'xai-grok-pager'),
    },
    {
        platform: 'win32', arch: 'x64', binName: 'grok.exe',
        envVar: 'GROK_WIN32_X64',
        defaultSource: path.join(
            repositoryRoot, 'target', 'x86_64-pc-windows-msvc',
            'release', 'xai-grok-pager.exe'),
    },
    {
        platform: 'win32', arch: 'arm64', binName: 'grok.exe',
        envVar: 'GROK_WIN32_ARM64',
        defaultSource: path.join(
            repositoryRoot, 'target', 'aarch64-pc-windows-msvc',
            'release', 'xai-grok-pager.exe'),
    },
]);

function usage() {
    return [
        'Usage:',
        '  node assemble-platform-packages.js --check-assets',
        '  node assemble-platform-packages.js [--output-root <directory>]',
    ].join('\n');
}

function parseArgs(argv) {
    let checkAssets = false;
    let outputRoot = npmRoot;
    let outputRootWasSet = false;

    for (let index = 0; index < argv.length; index++) {
        const argument = argv[index];
        if (argument === '--check-assets') {
            if (checkAssets) throw new Error('--check-assets may be specified only once');
            checkAssets = true;
        } else if (argument === '--output-root') {
            if (outputRootWasSet) throw new Error('--output-root may be specified only once');
            const value = argv[++index];
            if (!value || value.startsWith('--')) {
                throw new Error('--output-root requires a directory argument');
            }
            outputRoot = path.resolve(value);
            outputRootWasSet = true;
        } else if (argument === '--help' || argument === '-h') {
            return { help: true, checkAssets: false, outputRoot: npmRoot };
        } else {
            throw new Error(`unknown argument: ${argument}`);
        }
    }
    if (checkAssets && outputRootWasSet) {
        throw new Error('--check-assets does not write output; do not combine it with --output-root');
    }
    return { help: false, checkAssets, outputRoot };
}

function sha256(data) {
    return crypto.createHash('sha256').update(data).digest('hex');
}

function readUtf8(data, label) {
    try {
        return utf8Decoder.decode(data);
    } catch (error) {
        throw new Error(`${label} is not valid UTF-8: ${error.message}`);
    }
}

function readRegularFile(file, label, maxBytes = Number.MAX_SAFE_INTEGER) {
    let metadata;
    try {
        metadata = fs.lstatSync(file);
    } catch (error) {
        throw new Error(`failed to inspect ${label} ${file}: ${error.message}`);
    }
    if (metadata.isSymbolicLink() || !metadata.isFile()) {
        throw new Error(`${label} must be a regular file, not a symlink or special file: ${file}`);
    }
    if (metadata.size > maxBytes) {
        throw new Error(`${label} is ${metadata.size} bytes; limit is ${maxBytes}: ${file}`);
    }
    const data = fs.readFileSync(file);
    if (data.length !== metadata.size) {
        throw new Error(`${label} changed while it was being read: ${file}`);
    }
    return data;
}

function readJsonFile(file, label, maxBytes = Number.MAX_SAFE_INTEGER) {
    const bytes = readRegularFile(file, label, maxBytes);
    const text = readUtf8(bytes, label);
    try {
        return { bytes, value: JSON.parse(text) };
    } catch (error) {
        throw new Error(`${label} is not valid JSON: ${error.message}`);
    }
}

function exactKeys(value, keys, label) {
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
        throw new Error(`${label} must be a JSON object`);
    }
    const actual = Object.keys(value).sort();
    const expected = [...keys].sort();
    if (JSON.stringify(actual) !== JSON.stringify(expected)) {
        throw new Error(
            `${label} keys must be ${expected.join(', ')}; got ${actual.join(', ')}`);
    }
}

function nonNegativeInteger(value, label) {
    if (!Number.isSafeInteger(value) || value < 0) {
        throw new Error(`${label} must be a non-negative safe integer`);
    }
    return value;
}

function validatePortableComponent(component, label) {
    if (!component || component === '.' || component === '..') {
        throw new Error(`${label} contains an empty or traversal component: ${JSON.stringify(component)}`);
    }
    if (component.normalize('NFC') !== component) {
        throw new Error(`${label} component is not Unicode NFC: ${JSON.stringify(component)}`);
    }
    if (BIDI_CONTROL_RE.test(component)) {
        throw new Error(`${label} component contains a bidi control: ${JSON.stringify(component)}`);
    }
    if (/[\u0000-\u001f\u007f]/.test(component)) {
        throw new Error(`${label} component contains a control character: ${JSON.stringify(component)}`);
    }
    if (WINDOWS_RESERVED_CHARACTER_RE.test(component)) {
        throw new Error(
            `${label} component contains a Windows-reserved character: ${JSON.stringify(component)}`);
    }
    if (component.endsWith('.') || component.endsWith(' ')) {
        throw new Error(
            `${label} component has a Windows-unsafe trailing dot or space: ` +
            JSON.stringify(component));
    }
    const windowsStem = component.split('.', 1)[0].toUpperCase();
    if (WINDOWS_RESERVED_NAMES.has(windowsStem)) {
        throw new Error(
            `${label} component uses a Windows-reserved device name: ${JSON.stringify(component)}`);
    }
    if (!PORTABLE_COMPONENT_RE.test(component)) {
        throw new Error(
            `${label} component must use only portable ASCII letters, digits, '.', '_', or '-': ` +
            JSON.stringify(component));
    }
    if (Buffer.byteLength(component, 'ascii') > MAX_PORTABLE_COMPONENT_BYTES) {
        throw new Error(
            `${label} component exceeds ${MAX_PORTABLE_COMPONENT_BYTES} bytes: ` +
            JSON.stringify(component));
    }
}

function safeRelativePath(value, label) {
    if (typeof value !== 'string' || value.length === 0) {
        throw new Error(`${label} must be a non-empty string path`);
    }
    if (Buffer.byteLength(value, 'utf8') > MAX_PORTABLE_PATH_BYTES) {
        throw new Error(`${label} exceeds ${MAX_PORTABLE_PATH_BYTES} bytes`);
    }
    if (value.startsWith('/') || /^[A-Za-z]:/.test(value) || value.includes('\\')) {
        throw new Error(`${label} contains an absolute or backslash path: ${JSON.stringify(value)}`);
    }
    for (const component of value.split('/')) {
        validatePortableComponent(component, label);
    }
    return value;
}

function themePathParts(value, label) {
    const safe = safeRelativePath(value, label);
    const parts = safe.split('/');
    if (parts.length !== 2 || !CATEGORIES.includes(parts[0]) || !/\.(?:yaml|yml)$/.test(parts[1])) {
        throw new Error(
            `${label} must be one lowercase YAML/YML file directly below an allowlisted category: ` +
            JSON.stringify(value));
    }
    const stem = parts[1].replace(/\.(?:yaml|yml)$/, '');
    if (!stem) throw new Error(`${label} has an empty theme stem: ${JSON.stringify(value)}`);
    return { category: parts[0], stem };
}

function validateHashRecord(record, label) {
    exactKeys(record, ['path', 'bytes', 'sha256'], label);
    const recordPath = safeRelativePath(record.path, `${label}.path`);
    const byteCount = nonNegativeInteger(record.bytes, `${label}.bytes`);
    if (typeof record.sha256 !== 'string' || !SHA256_RE.test(record.sha256)) {
        throw new Error(`${label}.sha256 must be 64 lowercase hexadecimal characters`);
    }
    return { path: recordPath, bytes: byteCount, sha256: record.sha256 };
}

function assertNoCaseCollisions(paths, label) {
    const seen = new Map();
    for (const item of paths) {
        const key = item.normalize('NFC').toLowerCase();
        const previous = seen.get(key);
        if (previous !== undefined) {
            if (previous === item) {
                throw new Error(`${label} contains duplicate entry: ${item}`);
            }
            throw new Error(`${label} has a case-insensitive collision: ${previous} and ${item}`);
        }
        seen.set(key, item);
    }
}

function validateWarpCorpus(root = warpThemesRoot) {
    const manifestSource = path.join(root, 'VENDOR_MANIFEST.json');
    const licenseSource = path.join(root, 'LICENSE');
    const revisionSource = path.join(root, 'UPSTREAM_REVISION');
    const readmeSource = path.join(root, 'README.md');
    const rootMetadata = fs.lstatSync(root);
    if (rootMetadata.isSymbolicLink() || !rootMetadata.isDirectory()) {
        throw new Error(`Warp themes root must be a real directory: ${root}`);
    }

    const rootEntries = fs.readdirSync(root, { withFileTypes: true });
    const observedRootFiles = [];
    const observedCategories = [];
    for (const entry of rootEntries) {
        safeRelativePath(entry.name, 'Warp themes root entry');
        const entryPath = path.join(root, entry.name);
        const metadata = fs.lstatSync(entryPath);
        if (metadata.isSymbolicLink()) {
            throw new Error(`Warp themes root rejects symlink ${entry.name}`);
        }
        if (metadata.isDirectory()) {
            if (!CATEGORIES.includes(entry.name)) {
                throw new Error(`Warp themes root rejects directory ${entry.name}`);
            }
            observedCategories.push(entry.name);
        } else if (metadata.isFile()) {
            if (!ROOT_FILES.includes(entry.name)) {
                throw new Error(`Warp themes root rejects file ${entry.name}`);
            }
            observedRootFiles.push(entry.name);
        } else {
            throw new Error(`Warp themes root rejects special entry ${entry.name}`);
        }
    }
    if (JSON.stringify(observedRootFiles.sort()) !== JSON.stringify([...ROOT_FILES].sort())) {
        throw new Error(
            `Warp themes root files differ from strict allowlist: ${observedRootFiles.join(', ')}`);
    }
    if (JSON.stringify(observedCategories.sort()) !== JSON.stringify([...CATEGORIES].sort())) {
        throw new Error(
            `Warp theme categories differ from strict allowlist: ${observedCategories.join(', ')}`);
    }

    const actualThemes = new Map();
    const actualCounts = Object.fromEntries(CATEGORIES.map(category => [category, 0]));
    const logicalIds = [];
    for (const category of CATEGORIES) {
        const categoryRoot = path.join(root, category);
        for (const entry of fs.readdirSync(categoryRoot, { withFileTypes: true })) {
            const relative = `${category}/${entry.name}`;
            const { stem } = themePathParts(relative, 'Warp filesystem theme');
            const themeFile = path.join(categoryRoot, entry.name);
            const metadata = fs.lstatSync(themeFile);
            if (metadata.isSymbolicLink() || !metadata.isFile()) {
                throw new Error(`Warp theme must be a regular file: ${relative}`);
            }
            const bytes = readRegularFile(themeFile, 'Warp theme', MAX_THEME_BYTES);
            if (bytes.length === 0) throw new Error(`Warp theme is empty: ${relative}`);
            readUtf8(bytes, `Warp theme ${relative}`);
            actualThemes.set(relative, bytes);
            actualCounts[category]++;
            logicalIds.push(`${category}/${stem}`);
        }
    }
    const actualPaths = [...actualThemes.keys()].sort();
    assertNoCaseCollisions(actualPaths, 'Warp filesystem paths');
    assertNoCaseCollisions(logicalIds, 'Warp filesystem theme IDs');
    if (actualPaths.length !== EXPECTED_THEME_COUNT) {
        throw new Error(
            `Warp filesystem has ${actualPaths.length} themes; expected ${EXPECTED_THEME_COUNT}`);
    }
    if (JSON.stringify(actualCounts) !== JSON.stringify(EXPECTED_CATEGORY_COUNTS)) {
        throw new Error(
            `Warp filesystem category counts ${JSON.stringify(actualCounts)} do not match ` +
            JSON.stringify(EXPECTED_CATEGORY_COUNTS));
    }

    const { bytes: manifestBytes, value: manifest } = readJsonFile(
        manifestSource, 'Warp vendor manifest', MAX_MANIFEST_BYTES);
    const manifestHash = sha256(manifestBytes);
    if (manifestHash !== EXPECTED_VENDOR_MANIFEST_SHA256) {
        throw new Error(
            `Warp vendor manifest SHA-256 ${manifestHash} does not match audited canonical ` +
            EXPECTED_VENDOR_MANIFEST_SHA256);
    }
    exactKeys(
        manifest,
        ['schema_version', 'source', 'revision', 'theme_count', 'category_counts', 'license', 'files'],
        'Warp vendor manifest');
    if (manifest.schema_version !== 1) {
        throw new Error(`Warp vendor manifest schema_version must be 1`);
    }
    if (manifest.source !== EXPECTED_WARP_SOURCE) {
        throw new Error(
            `Warp vendor manifest source must be ${EXPECTED_WARP_SOURCE}; got ${manifest.source}`);
    }
    if (typeof manifest.revision !== 'string' || !REVISION_RE.test(manifest.revision)) {
        throw new Error('Warp vendor manifest revision must be an exact 40-character lowercase commit ID');
    }
    if (manifest.revision !== EXPECTED_WARP_REVISION) {
        throw new Error(
            `Warp vendor manifest revision ${manifest.revision} does not match audited revision ` +
            EXPECTED_WARP_REVISION);
    }
    if (nonNegativeInteger(manifest.theme_count, 'Warp manifest theme_count') !== EXPECTED_THEME_COUNT) {
        throw new Error(
            `Warp vendor manifest theme_count must be ${EXPECTED_THEME_COUNT}; got ` +
            manifest.theme_count);
    }
    exactKeys(manifest.category_counts, CATEGORIES, 'Warp manifest category_counts');
    for (const category of CATEGORIES) {
        if (nonNegativeInteger(
            manifest.category_counts[category],
            `Warp manifest category_counts.${category}`) !== EXPECTED_CATEGORY_COUNTS[category]) {
            throw new Error(
                `Warp manifest category count for ${category} must be ` +
                `${EXPECTED_CATEGORY_COUNTS[category]}; got ${manifest.category_counts[category]}`);
        }
    }

    const licenseRecord = validateHashRecord(manifest.license, 'Warp manifest license');
    if (licenseRecord.path !== 'LICENSE') {
        throw new Error('Warp manifest license path must be exactly LICENSE');
    }
    if (licenseRecord.sha256 !== EXPECTED_WARP_LICENSE_SHA256) {
        throw new Error(
            `Warp manifest license SHA-256 ${licenseRecord.sha256} does not match audited ` +
            EXPECTED_WARP_LICENSE_SHA256);
    }
    const licenseBytes = readRegularFile(
        licenseSource, 'Warp themes license', MAX_LICENSE_BYTES);
    readUtf8(licenseBytes, 'Warp themes license');
    if (licenseBytes.length !== licenseRecord.bytes || sha256(licenseBytes) !== licenseRecord.sha256) {
        throw new Error('Warp themes license bytes/hash do not match VENDOR_MANIFEST.json');
    }

    if (!Array.isArray(manifest.files)) {
        throw new Error('Warp vendor manifest files must be an array');
    }
    const manifestPaths = [];
    const manifestCounts = Object.fromEntries(CATEGORIES.map(category => [category, 0]));
    const canonicalRecords = [];
    for (let index = 0; index < manifest.files.length; index++) {
        const record = validateHashRecord(
            manifest.files[index], `Warp manifest files[${index}]`);
        const { category } = themePathParts(record.path, 'Warp manifest theme');
        if (record.bytes === 0 || record.bytes > MAX_THEME_BYTES) {
            throw new Error(`Warp manifest size is outside 1..${MAX_THEME_BYTES}: ${record.path}`);
        }
        const bytes = actualThemes.get(record.path);
        if (bytes === undefined) {
            throw new Error(`Warp manifest records missing filesystem theme: ${record.path}`);
        }
        if (bytes.length !== record.bytes || sha256(bytes) !== record.sha256) {
            throw new Error(`Warp theme bytes/hash do not match manifest: ${record.path}`);
        }
        manifestPaths.push(record.path);
        manifestCounts[category]++;
        canonicalRecords.push(record);
    }
    if (JSON.stringify(manifestPaths) !== JSON.stringify([...manifestPaths].sort())) {
        throw new Error('Warp manifest file records must be sorted by path');
    }
    if (new Set(manifestPaths).size !== manifestPaths.length) {
        throw new Error('Warp manifest contains duplicate paths');
    }
    assertNoCaseCollisions(manifestPaths, 'Warp manifest paths');
    if (JSON.stringify(manifestPaths) !== JSON.stringify(actualPaths)) {
        throw new Error('Warp manifest and filesystem path sets differ');
    }
    if (JSON.stringify(manifestCounts) !== JSON.stringify(EXPECTED_CATEGORY_COUNTS)) {
        throw new Error('Warp manifest file paths do not yield audited category counts');
    }

    const revisionBytes = readRegularFile(revisionSource, 'Warp revision marker', 256);
    const expectedRevisionBytes = Buffer.from(`${EXPECTED_WARP_REVISION}\n`, 'ascii');
    if (!revisionBytes.equals(expectedRevisionBytes)) {
        throw new Error('UPSTREAM_REVISION must contain exactly the audited revision and one newline');
    }
    readUtf8(readRegularFile(readmeSource, 'Warp themes README', 256 * 1024),
        'Warp themes README');

    const canonicalManifest = {
        schema_version: 1,
        source: EXPECTED_WARP_SOURCE,
        revision: EXPECTED_WARP_REVISION,
        theme_count: EXPECTED_THEME_COUNT,
        category_counts: { ...EXPECTED_CATEGORY_COUNTS },
        license: licenseRecord,
        files: canonicalRecords,
    };
    const canonicalBytes = Buffer.from(`${JSON.stringify(canonicalManifest, null, 2)}\n`, 'utf8');
    if (!manifestBytes.equals(canonicalBytes)) {
        throw new Error('VENDOR_MANIFEST.json is not in canonical generated form');
    }
    return { licenseBytes, manifest, actualThemes };
}

function validatePackageTemplate(target, meta) {
    const packageName = `grok-${target.platform}-${target.arch}`;
    const packageRoot = path.join(npmRoot, packageName);
    const metadata = fs.lstatSync(packageRoot);
    if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
        throw new Error(`per-platform package template must be a real directory: ${packageRoot}`);
    }
    const packageJsonPath = path.join(packageRoot, 'package.json');
    const { bytes: packageJsonBytes, value: packageJson } = readJsonFile(
        packageJsonPath, `${packageName} package.json`, 256 * 1024);
    const dependencyName = `@xai-official/${packageName}`;
    if (packageJson.name !== dependencyName) {
        throw new Error(
            `${packageName} package.json name must be ${dependencyName}; got ${packageJson.name}`);
    }
    if (JSON.stringify(packageJson.os) !== JSON.stringify([target.platform]) ||
        JSON.stringify(packageJson.cpu) !== JSON.stringify([target.arch])) {
        throw new Error(
            `${packageName} package.json os/cpu must be ${target.platform}/${target.arch}`);
    }
    const expectedFiles = ['bin/', NOTICES_NAME, WARP_THEMES_LICENSE_NAME].sort();
    const actualFiles = Array.isArray(packageJson.files) ? [...packageJson.files].sort() : [];
    if (JSON.stringify(actualFiles) !== JSON.stringify(expectedFiles)) {
        throw new Error(
            `${packageName} package.json files must be exactly ${expectedFiles.join(', ')}`);
    }
    if (packageJson.license !== 'Apache-2.0') {
        throw new Error(`${packageName} package.json license must be Apache-2.0`);
    }
    if (!meta.optionalDependencies || meta.optionalDependencies[dependencyName] !== meta.version) {
        throw new Error(
            `meta package optional dependency ${dependencyName} must be pinned to ${meta.version}`);
    }
    const templateFiles = new Map([
        ['.gitignore', readRegularFile(
            path.join(packageRoot, '.gitignore'), `${packageName} template .gitignore`,
            1024 * 1024)],
        ['README.md', readRegularFile(
            path.join(packageRoot, 'README.md'), `${packageName} template README.md`,
            1024 * 1024)],
        ['package.json', packageJsonBytes],
    ]);
    return { packageName, packageRoot, templateFiles };
}

function validateNoticeSources(sources = NOTICES_SOURCES) {
    const noticeBuffers = [];
    for (const { source, bytes: expectedBytes, sha256: expectedHash, requiredMarker } of sources) {
        const bytes = readRegularFile(source, 'third-party notices', 8 * 1024 * 1024);
        const actualHash = sha256(bytes);
        if (bytes.length !== expectedBytes || actualHash !== expectedHash) {
            throw new Error(
                `third-party notices integrity mismatch for ${source}: ` +
                `${bytes.length} bytes / ${actualHash}; expected ` +
                `${expectedBytes} bytes / ${expectedHash}`);
        }
        const text = readUtf8(bytes, `third-party notices ${source}`);
        if (!text.includes(requiredMarker)) {
            throw new Error(
                `third-party notices file is missing required marker ${requiredMarker}: ${source}`);
        }
        noticeBuffers.push(bytes);
    }
    const separator = Buffer.from('\n---\n\n', 'utf8');
    const combinedParts = [];
    noticeBuffers.forEach((bytes, index) => {
        if (index > 0) combinedParts.push(separator);
        combinedParts.push(bytes);
    });
    const combinedNotices = Buffer.concat(combinedParts);
    const combinedText = readUtf8(combinedNotices, 'combined third-party notices');
    for (const { requiredMarker } of sources) {
        if (!combinedText.includes(requiredMarker)) {
            throw new Error(`combined third-party notices omitted marker ${requiredMarker}`);
        }
    }
    return combinedNotices;
}

function validateAssets({ warpRoot = warpThemesRoot, noticeSources = NOTICES_SOURCES } = {}) {
    const { value: meta } = readJsonFile(META_PKG_JSON, 'meta package.json', 256 * 1024);
    if (typeof meta.version !== 'string' || !meta.version) {
        throw new Error('meta package.json must contain a non-empty version');
    }

    const warp = validateWarpCorpus(warpRoot);
    const combinedNotices = validateNoticeSources(noticeSources);
    const templates = TARGETS.map(target => validatePackageTemplate(target, meta));
    return {
        version: meta.version,
        warpLicense: warp.licenseBytes,
        combinedNotices,
        templates,
    };
}

function binaryOpenFlags() {
    const { O_RDONLY, O_NOFOLLOW, O_NONBLOCK, O_CLOEXEC } = fs.constants;
    if (!Number.isInteger(O_NOFOLLOW) || O_NOFOLLOW === 0 ||
        !Number.isInteger(O_NONBLOCK) || O_NONBLOCK === 0) {
        throw new Error(
            'this platform cannot guarantee no-follow, non-blocking binary preflight; refusing assembly');
    }
    let flags = O_RDONLY | O_NOFOLLOW | O_NONBLOCK;
    if (Number.isInteger(O_CLOEXEC)) flags |= O_CLOEXEC;
    return flags;
}

function readBinaryNoFollow(source, label) {
    const flags = binaryOpenFlags();
    let descriptor;
    try {
        descriptor = fs.openSync(source, flags);
    } catch (error) {
        const detail = error && error.code === 'ENOENT' ? 'missing' : error.message;
        throw new Error(`${label} could not be opened without following links: ${source}: ${detail}`);
    }
    try {
        const before = fs.fstatSync(descriptor, { bigint: true });
        if (!before.isFile()) {
            throw new Error(`${label} must be a regular file: ${source}`);
        }
        if (before.size > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error(`${label} is too large to read safely: ${source}`);
        }
        const bytes = fs.readFileSync(descriptor);
        const after = fs.fstatSync(descriptor, { bigint: true });
        const stable = before.dev === after.dev &&
            before.ino === after.ino &&
            before.size === after.size &&
            before.mtimeNs === after.mtimeNs &&
            before.ctimeNs === after.ctimeNs;
        if (!stable || BigInt(bytes.length) !== before.size) {
            throw new Error(`${label} changed while it was read from one descriptor: ${source}`);
        }
        return bytes;
    } finally {
        fs.closeSync(descriptor);
    }
}

function preflightBinarySources(targets = TARGETS, environment = process.env) {
    return targets.map(target => {
        const source = environment[target.envVar] || target.defaultSource;
        const label = `binary for ${target.platform}-${target.arch}`;
        let binaryBytes;
        try {
            binaryBytes = readBinaryNoFollow(source, label);
        } catch (error) {
            throw new Error(
                `${error.message}\nSet ${target.envVar} or build to the default location.`);
        }
        return { ...target, source, binaryBytes };
    });
}

function stagePackageTemplate(template, stagingRoot) {
    const stagedPackageRoot = path.join(stagingRoot, template.packageName);
    fs.mkdirSync(path.join(stagedPackageRoot, 'bin'), { recursive: true });
    // Copy only human-maintained template inputs. Ignored binaries, Brotli
    // outputs, and legal artifacts from an earlier in-tree assembly can never
    // leak into a new output root. All bytes were preflighted before the
    // output root was created, so staging performs no further source reads.
    for (const [name, bytes] of template.templateFiles) {
        fs.writeFileSync(path.join(stagedPackageRoot, name), bytes);
    }
    return stagedPackageRoot;
}

async function packPlatform(target, template, stagedPackageRoot, assets, outputRoot) {
    const packageJsonPath = path.join(stagedPackageRoot, 'package.json');
    const packageJson = JSON.parse(readUtf8(
        readRegularFile(packageJsonPath, `${template.packageName} staged package.json`, 256 * 1024),
        `${template.packageName} staged package.json`));
    packageJson.version = assets.version;
    fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 4)}\n`);

    // Preserve all source notice/license bytes exactly. The only new bytes in
    // the combined notice are separators between complete source documents.
    fs.writeFileSync(path.join(stagedPackageRoot, NOTICES_NAME), assets.combinedNotices);
    fs.writeFileSync(path.join(stagedPackageRoot, WARP_THEMES_LICENSE_NAME), assets.warpLicense);

    const outputBrotli = path.join(stagedPackageRoot, 'bin', `${target.binName}.br`);
    fs.mkdirSync(path.dirname(outputBrotli), { recursive: true });
    const raw = target.binaryBytes;
    const compressed = await brotliCompress(raw, {
        params: { [zlib.constants.BROTLI_PARAM_QUALITY]: zlib.constants.BROTLI_MAX_QUALITY },
    });
    fs.writeFileSync(outputBrotli, compressed);
    const publishedBrotli = path.join(
        outputRoot, template.packageName, 'bin', `${target.binName}.br`);
    return (
        `[assemble] ${template.packageName}@${assets.version}: ` +
        `${(raw.length / 1048576).toFixed(1)} MB -> ` +
        `${(compressed.length / 1048576).toFixed(1)} MB ` +
        `(${path.relative(outputRoot, publishedBrotli)})`
    );
}

function errorMessage(error) {
    return error instanceof Error ? error.message : String(error);
}

function publishStagedPackages(stagingRoot, outputRoot, templates, fileOps = fs) {
    const published = [];
    try {
        for (const template of templates) {
            const staged = path.join(stagingRoot, template.packageName);
            const destination = path.join(outputRoot, template.packageName);
            const backup = path.join(
                outputRoot, `.${template.packageName}.backup-${process.pid}-${crypto.randomUUID()}`);
            let hadDestination = false;
            if (fileOps.existsSync(destination)) {
                const metadata = fileOps.lstatSync(destination);
                if (metadata.isSymbolicLink() || !metadata.isDirectory()) {
                    throw new Error(`output package path must be a real directory: ${destination}`);
                }
                fileOps.renameSync(destination, backup);
                hadDestination = true;
            }
            try {
                fileOps.renameSync(staged, destination);
            } catch (installError) {
                if (hadDestination) {
                    try {
                        fileOps.renameSync(backup, destination);
                    } catch (restoreError) {
                        throw new Error(
                            `failed to publish ${template.packageName}: ` +
                            `${errorMessage(installError)}; failed to restore previous package, ` +
                            `which remains at ${backup}: ${errorMessage(restoreError)}`);
                    }
                }
                throw new Error(
                    `failed to publish ${template.packageName}: ${errorMessage(installError)}`);
            }
            published.push({ destination, backup, hadDestination });
        }
    } catch (publishError) {
        const rollbackErrors = [];
        for (const item of [...published].reverse()) {
            try {
                fileOps.rmSync(item.destination, { recursive: true, force: true });
            } catch (removeError) {
                rollbackErrors.push(
                    `could not remove new package ${item.destination}: ${errorMessage(removeError)}`);
                continue;
            }
            if (item.hadDestination) {
                if (!fileOps.existsSync(item.backup)) {
                    rollbackErrors.push(
                        `could not restore ${item.destination}; previous package backup is missing: ` +
                        item.backup);
                    continue;
                }
                try {
                    fileOps.renameSync(item.backup, item.destination);
                } catch (restoreError) {
                    rollbackErrors.push(
                        `could not restore ${item.destination}; previous package remains at ` +
                        `${item.backup}: ${errorMessage(restoreError)}`);
                }
            }
        }
        if (rollbackErrors.length > 0) {
            throw new Error(
                `${errorMessage(publishError)}; publication rollback incomplete: ` +
                rollbackErrors.join('; '));
        }
        throw publishError;
    }

    const cleanupErrors = [];
    for (const item of published) {
        if (item.hadDestination && fileOps.existsSync(item.backup)) {
            try {
                fileOps.rmSync(item.backup, { recursive: true, force: true });
            } catch (cleanupError) {
                cleanupErrors.push(
                    `previous package backup remains at ${item.backup}: ` +
                    errorMessage(cleanupError));
            }
        }
    }
    if (cleanupErrors.length > 0) {
        throw new Error(
            `packages were published, but backup cleanup was incomplete: ` +
            cleanupErrors.join('; '));
    }
}

function realpathNative(value) {
    return fs.realpathSync.native ? fs.realpathSync.native(value) : fs.realpathSync(value);
}

function conservativePathKey(value) {
    let resolved = path.resolve(value).normalize('NFC').toUpperCase().toLowerCase();
    const root = path.parse(resolved).root;
    while (resolved.length > root.length && resolved.endsWith(path.sep)) {
        resolved = resolved.slice(0, -path.sep.length);
    }
    return resolved;
}

function pathIsWithin(candidate, parent) {
    const candidateKey = conservativePathKey(candidate);
    const parentKey = conservativePathKey(parent);
    return candidateKey === parentKey || candidateKey.startsWith(`${parentKey}${path.sep}`);
}

function inspectDirectoryPath(input, label, { requireExists = false } = {}) {
    const absolute = path.resolve(input);
    const parsed = path.parse(absolute);
    const components = absolute
        .slice(parsed.root.length)
        .split(path.sep)
        .filter(Boolean);
    let current = parsed.root;
    let deepestExisting = parsed.root;
    const missing = [];
    let missingStarted = false;

    for (const component of components) {
        current = path.join(current, component);
        if (missingStarted) {
            missing.push(component);
            continue;
        }
        let metadata;
        try {
            metadata = fs.lstatSync(current);
        } catch (error) {
            if (error && error.code === 'ENOENT') {
                missingStarted = true;
                missing.push(component);
                continue;
            }
            throw new Error(`failed to inspect ${label} ancestor ${current}: ${error.message}`);
        }
        if (metadata.isSymbolicLink()) {
            throw new Error(`${label} rejects symlink or junction ancestor: ${current}`);
        }
        if (!metadata.isDirectory()) {
            throw new Error(`${label} ancestor must be a directory: ${current}`);
        }
        deepestExisting = current;
    }

    if (requireExists && missing.length > 0) {
        throw new Error(`${label} does not exist after creation: ${absolute}`);
    }
    const canonicalExisting = realpathNative(deepestExisting);
    if (conservativePathKey(canonicalExisting) !== conservativePathKey(deepestExisting)) {
        throw new Error(
            `${label} existing ancestor resolves through a symlink or junction: ` +
            `${deepestExisting} -> ${canonicalExisting}`);
    }
    return {
        absolute,
        canonical: path.join(canonicalExisting, ...missing),
        exists: missing.length === 0,
    };
}

function rejectTemplateContainment(candidate) {
    for (const target of TARGETS) {
        const templateRoot = path.join(npmRoot, `grok-${target.platform}-${target.arch}`);
        const inspectedTemplate = inspectDirectoryPath(
            templateRoot, 'package template', { requireExists: true });
        if (pathIsWithin(candidate, inspectedTemplate.canonical)) {
            throw new Error(
                `--output-root may not be or be nested inside package template ${templateRoot}`);
        }
    }
}

function ensureOutputRoot(outputRoot, hooks = {}) {
    const before = inspectDirectoryPath(outputRoot, '--output-root');
    rejectTemplateContainment(before.canonical);
    if (!before.exists) {
        fs.mkdirSync(before.absolute, { recursive: true });
    }
    if (hooks.afterCreate) hooks.afterCreate(before.absolute);

    const after = inspectDirectoryPath(
        before.absolute, '--output-root', { requireExists: true });
    if (conservativePathKey(after.canonical) !== conservativePathKey(before.canonical)) {
        throw new Error(
            `--output-root canonical path changed during creation: ` +
            `${before.canonical} -> ${after.canonical}`);
    }
    rejectTemplateContainment(after.canonical);
    return after.canonical;
}

async function assemble(outputRoot) {
    // Complete every read-only preflight before creating output. A missing
    // binary or corrupt legal/theme asset therefore leaves output untouched.
    const assets = validateAssets();
    const sources = preflightBinarySources();
    const safeOutputRoot = ensureOutputRoot(outputRoot);

    const stagingRoot = fs.mkdtempSync(
        path.join(safeOutputRoot, '.assemble-platform-packages-'));
    try {
        const stagedRoots = assets.templates.map(
            template => stagePackageTemplate(template, stagingRoot));
        const summaries = await Promise.all(sources.map((target, index) => packPlatform(
            target, assets.templates[index], stagedRoots[index], assets, safeOutputRoot)));
        for (const summary of summaries) console.log(summary);
        publishStagedPackages(stagingRoot, safeOutputRoot, assets.templates);
    } finally {
        fs.rmSync(stagingRoot, { recursive: true, force: true });
    }
    console.log(
        `[assemble] All ${TARGETS.length} per-platform packages assembled at version ` +
        `${assets.version} below ${safeOutputRoot}.`);
}

async function main(argv = process.argv.slice(2)) {
    const options = parseArgs(argv);
    if (options.help) {
        console.log(usage());
        return;
    }
    if (options.checkAssets) {
        const assets = validateAssets();
        console.log(
            `[assemble] Asset check passed: ${EXPECTED_THEME_COUNT} Warp themes at ` +
            `${EXPECTED_WARP_REVISION}; ${assets.templates.length} package templates.`);
        return;
    }
    await assemble(options.outputRoot);
}

if (require.main === module) {
    main().catch(error => {
        console.error(`[assemble] ${error.message}`);
        process.exitCode = 1;
    });
}

module.exports = {
    EXPECTED_VENDOR_MANIFEST_SHA256,
    EXPECTED_WARP_REVISION,
    NOTICES_SOURCES,
    TARGETS,
    assemble,
    ensureOutputRoot,
    main,
    npmRoot,
    parseArgs,
    preflightBinarySources,
    publishStagedPackages,
    readBinaryNoFollow,
    safeRelativePath,
    validateAssets,
    validateNoticeSources,
    validateWarpCorpus,
    warpThemesRoot,
};
