#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const test = require('node:test');
const zlib = require('node:zlib');

const assembler = require('./assemble-platform-packages.js');
const SCRIPT = path.join(__dirname, 'assemble-platform-packages.js');
const TEMPLATE_ROOTS = assembler.TARGETS.map(target => path.join(
    assembler.npmRoot, `grok-${target.platform}-${target.arch}`));
const PAGER_RENDER_ROOT = path.resolve(
    assembler.npmRoot, '..', '..', 'xai-grok-pager-render');
const WARP_LICENSE = path.join(PAGER_RENDER_ROOT, 'assets', 'warp-themes', 'LICENSE');
const META_PACKAGE_JSON = path.resolve(__dirname, '..', 'package.json');
const CANONICAL_TMP = fs.realpathSync(os.tmpdir());
const SECURE_BINARY_OPEN_SUPPORTED =
    Number.isInteger(fs.constants.O_NOFOLLOW) && fs.constants.O_NOFOLLOW !== 0 &&
    Number.isInteger(fs.constants.O_NONBLOCK) && fs.constants.O_NONBLOCK !== 0;

function hash(data) {
    return crypto.createHash('sha256').update(data).digest('hex');
}

function temporaryRoot(prefix) {
    return fs.mkdtempSync(path.join(CANONICAL_TMP, prefix));
}

function snapshotDirectory(root) {
    const snapshot = {};
    function visit(current, relative) {
        const metadata = fs.lstatSync(current);
        const key = relative || '.';
        if (metadata.isSymbolicLink()) {
            snapshot[key] = `symlink:${fs.readlinkSync(current)}`;
            return;
        }
        if (metadata.isDirectory()) {
            snapshot[key] = `directory:${metadata.mode & 0o777}`;
            for (const name of fs.readdirSync(current).sort()) {
                visit(path.join(current, name), relative ? `${relative}/${name}` : name);
            }
            return;
        }
        if (metadata.isFile()) {
            const bytes = fs.readFileSync(current);
            snapshot[key] = `file:${metadata.mode & 0o777}:${bytes.length}:${hash(bytes)}`;
            return;
        }
        snapshot[key] = `special:${metadata.mode}`;
    }
    visit(root, '');
    return snapshot;
}

function snapshotTemplates() {
    return Object.fromEntries(TEMPLATE_ROOTS.map(root => [root, snapshotDirectory(root)]));
}

function runAssembler(arguments_, environment = {}) {
    return spawnSync(process.execPath, [SCRIPT, ...arguments_], {
        encoding: 'utf8',
        env: { ...process.env, ...environment },
        maxBuffer: 4 * 1024 * 1024,
    });
}

function assertSucceeded(result) {
    assert.equal(
        result.status,
        0,
        `assembler failed\nstdout:\n${result.stdout}\nstderr:\n${result.stderr}`);
}

function fakeBinaryEnvironment(root) {
    const binariesRoot = path.join(root, 'binaries');
    fs.mkdirSync(binariesRoot);
    const environment = {};
    const expected = new Map();
    for (const target of assembler.TARGETS) {
        const key = `${target.platform}-${target.arch}`;
        const binary = Buffer.from(`deterministic fake binary for ${key}\n`, 'utf8');
        const binaryPath = path.join(binariesRoot, key);
        fs.writeFileSync(binaryPath, binary);
        environment[target.envVar] = binaryPath;
        expected.set(key, binary);
    }
    return { environment, expected };
}

function preparePublication(root, names) {
    const outputRoot = path.join(root, 'output');
    const stagingRoot = path.join(root, 'staging');
    fs.mkdirSync(outputRoot);
    fs.mkdirSync(stagingRoot);
    const templates = names.map(packageName => ({ packageName }));
    for (const packageName of names) {
        const destination = path.join(outputRoot, packageName);
        const staged = path.join(stagingRoot, packageName);
        fs.mkdirSync(destination);
        fs.mkdirSync(staged);
        fs.writeFileSync(path.join(destination, 'marker'), `old-${packageName}\n`);
        fs.writeFileSync(path.join(staged, 'marker'), `new-${packageName}\n`);
    }
    return { outputRoot, stagingRoot, templates };
}

test('--check-assets validates without mutating package templates', () => {
    const before = snapshotTemplates();
    const result = runAssembler(['--check-assets']);
    assertSucceeded(result);
    assert.match(result.stdout, /Asset check passed: 340 Warp themes/);
    assert.deepEqual(snapshotTemplates(), before);
});

test('canonical Warp manifest digest rejects otherwise parseable tampering', () => {
    const root = temporaryRoot('grok-manifest-tamper-test-');
    try {
        const copied = path.join(root, 'warp-themes');
        fs.cpSync(assembler.warpThemesRoot, copied, { recursive: true, dereference: false });
        const manifest = path.join(copied, 'VENDOR_MANIFEST.json');
        fs.appendFileSync(manifest, '\n');
        assert.throws(
            () => assembler.validateWarpCorpus(copied),
            /manifest SHA-256 .* does not match audited canonical/);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('third-party notice integrity is hash-owned, not marker-only', () => {
    const root = temporaryRoot('grok-notice-tamper-test-');
    try {
        const original = assembler.NOTICES_SOURCES[0];
        const tampered = path.join(root, 'THIRD_PARTY_NOTICES.md');
        fs.copyFileSync(original.source, tampered);
        fs.appendFileSync(tampered, '\nmarker-preserving tamper\n');
        const sources = [
            { ...original, source: tampered },
            ...assembler.NOTICES_SOURCES.slice(1),
        ];
        assert.throws(
            () => assembler.validateNoticeSources(sources),
            /third-party notices integrity mismatch/);
        assert.deepEqual(
            assembler.validateNoticeSources(), assembler.validateNoticeSources(),
            'legal bundle generation must be reproducible');
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('portable path checks reject hostile cross-platform names', () => {
    for (const rejected of [
        'base16/cafe\u0301.yaml',
        'base16/caf\u00e9.yaml',
        'base16/evil\u202eyaml.yaml',
        'base16/CON.yaml',
        'base16/nul.txt.yaml',
        'base16/has:colon.yaml',
        'base16/has space.yaml',
        'base16/trailing.',
        'C:/base16/theme.yaml',
        'base16\\theme.yaml',
    ]) {
        assert.throws(() => assembler.safeRelativePath(rejected, 'test path'));
    }
    assert.equal(
        assembler.safeRelativePath('base16/base16_3024.yaml', 'test path'),
        'base16/base16_3024.yaml');
});

test('binary preflight reads regular files and rejects leaf symlinks or unsupported hosts', () => {
    const root = temporaryRoot('grok-binary-nofollow-test-');
    try {
        const binary = path.join(root, 'binary');
        fs.writeFileSync(binary, 'binary bytes\n');
        if (!SECURE_BINARY_OPEN_SUPPORTED) {
            assert.throws(
                () => assembler.readBinaryNoFollow(binary, 'test binary'),
                /cannot guarantee no-follow/);
            return;
        }
        assert.deepEqual(
            assembler.readBinaryNoFollow(binary, 'test binary'),
            Buffer.from('binary bytes\n'));
        const linked = path.join(root, 'linked-binary');
        fs.symlinkSync(binary, linked, 'file');
        assert.throws(
            () => assembler.readBinaryNoFollow(linked, 'test binary'),
            /could not be opened without following links/);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('--output-root is reproducible and does not mutate templates', {
    skip: !SECURE_BINARY_OPEN_SUPPORTED,
}, () => {
    const root = temporaryRoot('grok-npm-assemble-test-');
    try {
        const outputRoot = path.join(root, 'output');
        const { environment, expected } = fakeBinaryEnvironment(root);
        const templatesBefore = snapshotTemplates();
        const first = runAssembler(['--output-root', outputRoot], environment);
        assertSucceeded(first);
        assert.deepEqual(snapshotTemplates(), templatesBefore);

        const meta = JSON.parse(fs.readFileSync(META_PACKAGE_JSON, 'utf8'));
        const expectedLicense = fs.readFileSync(WARP_LICENSE);
        for (const target of assembler.TARGETS) {
            const key = `${target.platform}-${target.arch}`;
            const packageName = `grok-${key}`;
            const packageRoot = path.join(outputRoot, packageName);
            const packageJson = JSON.parse(
                fs.readFileSync(path.join(packageRoot, 'package.json'), 'utf8'));
            assert.equal(packageJson.version, meta.version);
            assert.ok(packageJson.files.includes('THIRD_PARTY_NOTICES.md'));
            assert.ok(packageJson.files.includes('WARP_THEMES_LICENSE'));
            assert.deepEqual(
                fs.readFileSync(path.join(packageRoot, 'WARP_THEMES_LICENSE')),
                expectedLicense,
                `${packageName} must preserve exact Warp license bytes`);

            const notices = fs.readFileSync(
                path.join(packageRoot, 'THIRD_PARTY_NOTICES.md'), 'utf8');
            assert.match(notices, /src\/implementations\/grok_build\/web_search\//);
            assert.match(notices, /src\/auth\/codex\//);
            assert.match(notices, /warpdotdev\/themes/);

            const compressed = fs.readFileSync(
                path.join(packageRoot, 'bin', `${target.binName}.br`));
            assert.deepEqual(
                zlib.brotliDecompressSync(compressed), expected.get(key),
                `${packageName} compressed binary must round-trip`);
        }
        assert.deepEqual(
            fs.readdirSync(outputRoot).sort(),
            assembler.TARGETS.map(
                target => `grok-${target.platform}-${target.arch}`).sort());

        const firstSnapshot = snapshotDirectory(outputRoot);
        const second = runAssembler(['--output-root', outputRoot], environment);
        assertSucceeded(second);
        assert.deepEqual(snapshotDirectory(outputRoot), firstSnapshot);
        assert.deepEqual(snapshotTemplates(), templatesBefore);
        assert.equal(second.stdout, first.stdout, 'assembly logs must also be reproducible');
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('binary preflight failure leaves --output-root absent and templates unchanged', () => {
    const root = temporaryRoot('grok-npm-preflight-test-');
    try {
        const outputRoot = path.join(root, 'must-not-exist');
        const environment = Object.fromEntries(assembler.TARGETS.map(target => [
            target.envVar,
            path.join(root, `missing-${target.platform}-${target.arch}`),
        ]));
        const before = snapshotTemplates();
        const result = runAssembler(['--output-root', outputRoot], environment);
        assert.notEqual(result.status, 0, 'assembly with missing binaries must fail');
        assert.match(
            result.stderr,
            SECURE_BINARY_OPEN_SUPPORTED ? /could not be opened.*missing/s : /cannot guarantee no-follow/);
        assert.equal(fs.existsSync(outputRoot), false, 'failed preflight must not create output root');
        assert.deepEqual(snapshotTemplates(), before);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('--output-root rejects template case aliases without mutation', () => {
    const template = TEMPLATE_ROOTS[0];
    const caseAlias = path.join(
        path.dirname(template), path.basename(template).toUpperCase());
    const before = snapshotTemplates();
    assert.throws(
        () => assembler.ensureOutputRoot(caseAlias),
        /may not be or be nested inside package template/);
    assert.deepEqual(snapshotTemplates(), before);
});

test('--output-root rejects symlink ancestors before creating output', () => {
    const root = temporaryRoot('grok-output-symlink-ancestor-test-');
    try {
        const real = path.join(root, 'real');
        const alias = path.join(root, 'alias');
        fs.mkdirSync(real);
        fs.symlinkSync(real, alias, 'dir');
        const output = path.join(alias, 'output');
        assert.throws(
            () => assembler.ensureOutputRoot(output),
            /rejects symlink or junction ancestor/);
        assert.equal(fs.existsSync(path.join(real, 'output')), false);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('--output-root post-create revalidation catches a symlink swap', () => {
    const root = temporaryRoot('grok-output-revalidation-test-');
    try {
        const output = path.join(root, 'output');
        const redirect = path.join(root, 'redirect');
        fs.mkdirSync(redirect);
        assert.throws(
            () => assembler.ensureOutputRoot(output, {
                afterCreate(created) {
                    fs.rmSync(created, { recursive: true });
                    fs.symlinkSync(redirect, created, 'dir');
                },
            }),
            /rejects symlink or junction ancestor/);
        assert.equal(fs.readdirSync(redirect).length, 0);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('publication failure rolls every already-published package back', () => {
    const root = temporaryRoot('grok-publication-rollback-test-');
    try {
        const names = ['one', 'two', 'three'];
        const state = preparePublication(root, names);
        const failedStaged = path.join(state.stagingRoot, 'three');
        const failedDestination = path.join(state.outputRoot, 'three');
        const fileOps = {
            ...fs,
            renameSync(source, destination) {
                if (source === failedStaged && destination === failedDestination) {
                    throw new Error('injected publication failure');
                }
                fs.renameSync(source, destination);
            },
        };
        assert.throws(
            () => assembler.publishStagedPackages(
                state.stagingRoot, state.outputRoot, state.templates, fileOps),
            /injected publication failure/);
        for (const name of names) {
            assert.equal(
                fs.readFileSync(path.join(state.outputRoot, name, 'marker'), 'utf8'),
                `old-${name}\n`);
        }
        assert.equal(
            fs.readdirSync(state.outputRoot).some(name => name.includes('.backup-')), false);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test('failed rollback preserves the prior package at a reported backup path', () => {
    const root = temporaryRoot('grok-publication-preservation-test-');
    try {
        const names = ['one', 'two'];
        const state = preparePublication(root, names);
        const failedStaged = path.join(state.stagingRoot, 'two');
        const failedDestination = path.join(state.outputRoot, 'two');
        const fileOps = {
            ...fs,
            renameSync(source, destination) {
                if (source === failedStaged && destination === failedDestination) {
                    throw new Error('injected install failure');
                }
                if (path.basename(source).startsWith('.two.backup-') &&
                    destination === failedDestination) {
                    throw new Error('injected restore failure');
                }
                fs.renameSync(source, destination);
            },
        };
        assert.throws(
            () => assembler.publishStagedPackages(
                state.stagingRoot, state.outputRoot, state.templates, fileOps),
            /previous package.*remains at/);
        assert.equal(
            fs.readFileSync(path.join(state.outputRoot, 'one', 'marker'), 'utf8'),
            'old-one\n');
        assert.equal(fs.existsSync(failedDestination), false);
        const backup = fs.readdirSync(state.outputRoot).find(
            name => name.startsWith('.two.backup-'));
        assert.ok(backup, 'the prior package backup must be preserved');
        assert.equal(
            fs.readFileSync(path.join(state.outputRoot, backup, 'marker'), 'utf8'),
            'old-two\n');
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
