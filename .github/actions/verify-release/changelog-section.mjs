#!/usr/bin/env node
// Prints the CHANGELOG.md section of one version, without its heading: the body of the GitHub
// release for that tag. Fails when the version has no section or the section is empty, so a tag
// can never be released without its changelog entry.
//
//   node <devkit>/actions/scripts/changelog-section.mjs 1.2.0 [CHANGELOG.md] > notes.md
//
// Expects Keep a Changelog headings: "## [1.2.0] - 2026-10-15". A leading "v" is ignored.

import { readFileSync } from 'node:fs';

const [rawVersion, file = 'CHANGELOG.md'] = process.argv.slice(2);
if (!rawVersion) {
	console.error('usage: changelog-section.mjs <version> [file]');
	process.exit(2);
}

const version = rawVersion.replace(/^v/, '');
const lines = readFileSync(file, 'utf8').split(/\r?\n/);
const heading = `## [${version}]`;
const start = lines.findIndex((line) => line.startsWith(heading));

if (start === -1) {
	console.error(`changelog-section: no "${heading}" in ${file}. Add it before tagging.`);
	process.exit(1);
}

let end = lines.findIndex((line, index) => index > start && /^## \[/.test(line));
if (end === -1) end = lines.length;

// Link reference definitions at the bottom ("[1.2.0]: https://...") are not part of the notes.
const body = lines
	.slice(start + 1, end)
	.filter((line) => !/^\[[^\]]+\]:\s/.test(line))
	.join('\n')
	.trim();

if (!body) {
	console.error(`changelog-section: the ${version} section in ${file} is empty`);
	process.exit(1);
}

process.stdout.write(`${body}\n`);
