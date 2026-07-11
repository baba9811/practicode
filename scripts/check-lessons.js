#!/usr/bin/env node

const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { readFileSync, writeFileSync } = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const manifestPath = path.join(root, "assets", "lessons", "review-manifest.json");
const programmingLanguages = [
  ["python", "python"],
  ["ts", "typescript"],
  ["java", "java"],
  ["rust", "rust"],
];
const uiLanguages = ["en", "ko", "ja", "zh", "es"];
const requiredFields = [
  "title",
  "concept",
  "worked_example",
  "common_mistakes",
  "self_check",
  "exercise_prompt",
  "objective",
  "language_delta",
  "prediction_prompt",
  "transfer_trap",
];

function readJson(filename) {
  return JSON.parse(readFileSync(filename, "utf8"));
}

function hashFile(filename) {
  return createHash("sha256").update(readFileSync(filename)).digest("hex");
}

function profileFor(programmingLanguage, uiLanguage) {
  return uiLanguage === "en" ? `en-${programmingLanguage}` : uiLanguage;
}

function expectedEvidence() {
  const catalogs = [];
  const sources = {};
  for (const [programmingLanguage, directory] of programmingLanguages) {
    const coursePath = path.join(root, "assets", "lessons", directory, "course.json");
    const course = readJson(coursePath);
    const lessonIds = course.lessons.map((lesson) => lesson.id);
    sources[programmingLanguage] = [
      ...new Set(course.lessons.flatMap((lesson) => lesson.refs)),
    ].sort();
    for (const uiLanguage of uiLanguages) {
      const relativePath = `assets/lessons/${directory}/${uiLanguage}.json`;
      const filename = path.join(root, relativePath);
      catalogs.push({
        path: relativePath,
        programming_language: programmingLanguage,
        ui_language: uiLanguage,
        sha256: hashFile(filename),
        lesson_count: lessonIds.length,
        lesson_ids: lessonIds,
        review_profile: profileFor(programmingLanguage, uiLanguage),
      });
    }
  }
  return { catalogs, sources };
}

function validateCatalogFile(evidence) {
  const catalog = readJson(path.join(root, evidence.path));
  assert.equal(catalog.schema_version, 1, `${evidence.path}: schema_version`);
  assert.equal(
    catalog.programming_language,
    evidence.programming_language,
    `${evidence.path}: programming_language`,
  );
  assert.equal(catalog.ui_language, evidence.ui_language, `${evidence.path}: ui_language`);
  assert.deepEqual(Object.keys(catalog.lessons), evidence.lesson_ids, `${evidence.path}: lesson order`);
  for (const [lessonId, copy] of Object.entries(catalog.lessons)) {
    assert.deepEqual(Object.keys(copy), requiredFields, `${evidence.path}:${lessonId}: fields`);
    for (const field of requiredFields.filter(
      (name) => name !== "common_mistakes" && name !== "self_check",
    )) {
      assert.equal(typeof copy[field], "string", `${evidence.path}:${lessonId}:${field}`);
      assert.ok(copy[field].trim(), `${evidence.path}:${lessonId}:${field}: empty`);
    }
    assert.ok(
      Array.isArray(copy.common_mistakes),
      `${evidence.path}:${lessonId}: common_mistakes type`,
    );
    assert.ok(Array.isArray(copy.self_check), `${evidence.path}:${lessonId}: self_check type`);
    assert.ok(copy.common_mistakes.length >= 2, `${evidence.path}:${lessonId}: common_mistakes`);
    assert.ok(copy.self_check.length >= 2, `${evidence.path}:${lessonId}: self_check`);
    assert.ok(
      [...copy.common_mistakes, ...copy.self_check].every(
        (item) => typeof item === "string" && item.trim(),
      ),
      `${evidence.path}:${lessonId}: empty review prompt`,
    );
  }
}

function validateReviewProfile(name, profile) {
  assert.ok(profile, `missing review profile: ${name}`);
  for (const role of ["author", "blind_verifier"]) {
    const review = profile[role];
    assert.equal(typeof review?.identity, "string", `${name}:${role}: identity`);
    assert.ok(review.identity.trim(), `${name}:${role}: empty identity`);
    assert.equal(review.verdict, "approved", `${name}:${role}: verdict`);
    assert.equal(
      review.open_high_severity_findings,
      0,
      `${name}:${role}: open high-severity findings`,
    );
  }
  assert.notEqual(
    profile.author.identity,
    profile.blind_verifier.identity,
    `${name}: self-approval is not independent review`,
  );
  assert.ok(Array.isArray(profile.disagreements), `${name}: disagreements`);
  for (const disagreement of profile.disagreements) {
    assert.equal(disagreement.status, "resolved", `${name}: unresolved disagreement`);
    assert.ok(disagreement.resolution?.trim(), `${name}: missing disagreement resolution`);
  }
  assert.equal(profile.resolution, "approved", `${name}: resolution`);
}

function validate(manifest) {
  assert.equal(manifest.schema_version, 1, "review manifest schema_version");
  assert.equal(manifest.content_version, "0.2.0", "review manifest content_version");
  assert.ok(manifest.review_profiles && typeof manifest.review_profiles === "object");
  const expected = expectedEvidence();
  assert.deepEqual(manifest.sources, expected.sources, "official source URL coverage changed");
  for (const [language, urls] of Object.entries(manifest.sources)) {
    assert.ok(urls.length > 0, `${language}: empty source URL set`);
    assert.ok(
      urls.every((url) => /^https:\/\//.test(url)),
      `${language}: source URLs must use HTTPS`,
    );
  }
  assert.deepEqual(manifest.catalogs, expected.catalogs, "catalog review hashes or coverage changed");

  const usedProfiles = new Set();
  let reviewedRecords = 0;
  for (const evidence of manifest.catalogs) {
    assert.match(evidence.sha256, /^[0-9a-f]{64}$/, `${evidence.path}: sha256`);
    validateCatalogFile(evidence);
    validateReviewProfile(
      evidence.review_profile,
      manifest.review_profiles[evidence.review_profile],
    );
    usedProfiles.add(evidence.review_profile);
    reviewedRecords += evidence.lesson_count;
  }
  assert.equal(manifest.catalogs.length, 20, "reviewed catalog count");
  assert.equal(reviewedRecords, 550, "reviewed localized lesson record count");
  assert.deepEqual(
    [...Object.keys(manifest.review_profiles)].sort(),
    [...usedProfiles].sort(),
    "unused or missing review profile",
  );
}

function main() {
  const manifest = readJson(manifestPath);
  if (process.argv.includes("--refresh")) {
    const expected = expectedEvidence();
    manifest.sources = expected.sources;
    manifest.catalogs = expected.catalogs;
    writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  }
  validate(manifest);
  console.log("lesson review manifest: 20 catalogs, 550 records, all hashes verified");
}

try {
  main();
} catch (error) {
  console.error(`lesson review manifest failed: ${error.message}`);
  process.exitCode = 1;
}
