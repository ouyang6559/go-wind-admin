#!/usr/bin/env node
// Update-mask allowlist parity check.
//
// A handful of edit forms still hand-write their update FieldMask as a
// literal — or as a named constant holding one — instead of deriving it
// with makeUpdateMask(). Each such string is an allowlist: "these fields,
// and only these fields, may be written". They drift silently in both
// directions: a form field added without updating the mask silently stops
// saving; a field added to the mask silently becomes writable; and one
// frontend can quietly diverge from the other two. The literals survive
// because each one deliberately excludes something — an immutable field,
// credentials that ride as dedicated request fields with empty-means-keep
// server semantics, or (in one case) a narrow toggle update distinct from
// the full form mask; the per-site reasons are recorded inline in the
// registry below.
//
// This check enforces two properties:
//
//   1. Pinned values: every registered site carries exactly its registered
//      mask — literals are compared per file as multisets (a missing,
//      duplicated, extra or altered literal fails), constant references as
//      sets, and named mask constants as exact values. The three
//      frontends' equivalent sites are pinned to the same canonical
//      string, so the mask CONTENT is parity-checked even where the
//      surrounding files differ.
//   2. Closed allowlist: no other updateMask literal, constant reference,
//      or `const updateMask =` binding may appear anywhere in the three
//      application source trees. New code derives masks with
//      makeUpdateMask() from transport/rest; adding a genuinely new
//      allowlist site is a conscious decision that must be registered
//      here.
//
// Sweep exclusions: api/generated/** (make-ts output whose
// `updateMask: undefined | wellKnownFieldMask` lines are type
// declarations, not assignments) and the makeUpdateMask identifier itself
// (the sanctioned generic form). The vue-vben framework packages and
// everything outside the three application source trees carry no
// updateMask usage (verified when this check was introduced).
//
// Usage: node scripts/check-update-mask-allowlists.mjs

import { existsSync, readFileSync, readdirSync } from 'node:fs';

// Canonical mask values: one entry per distinct allowlist. Every site
// carrying a given allowlist — inline literal or named constant, in any of
// the three frontends — is pinned to the same canonical string, which is
// what makes the three frontends' mask content comparable.
const MASKS = {
  // Notification channel form: the editable fields of the channel data
  // object. The SMTP/webhook credentials ride as dedicated request fields
  // (password / webhookSecret) with empty-means-keep server semantics and
  // must never enter the mask.
  notificationChannel:
    'name,type,smtpHost,smtpPort,smtpUsername,smtpFrom,smtpTls,webhookUrl,webhookSignStyle,webhookPayloadTemplate,enabled,remark',
  // vben-local inline enable switch on the channel page: a narrow update
  // that only toggles enabled. react and vue-element edit enabled through
  // the drawer form instead. A deliberate three-frontend difference,
  // pinned as-is.
  notificationChannelEnabledToggle: 'enabled',
  // Notification rule form. event_type is deliberately excluded: it is
  // immutable after creation, and the edit form disables that column on
  // all three frontends.
  notificationRule: 'channel,isAsync,isEnabled,remark',
  // Notification template form. code is deliberately excluded: immutable
  // after creation.
  notificationTemplate:
    'name,titleTemplate,contentTemplate,isEnabled,remark',
  // Monitor alert form, full editable field set, pinned as-is on all
  // three frontends.
  monitorAlert:
    'name,metric,op,threshold,cooldownMinutes,channel,target,isEnabled,remark',
};

// Registered allowlist sites. Each entry pins, for one file, the complete
// set of updateMask usages that file may contain:
//   expectStrings — every literal mask value in the file, compared as a
//     multiset (counts included: a duplicated literal fails too)
//   expectIdents  — every constant-referenced mask in the file
//   defs          — named mask constants DEFINED in the file, pinned to
//     their exact value (usage and definition may live in different
//     files across the three frontends)
const REGISTRY = [
  // --- notification channel ---
  {
    // react drawer form, inline literal.
    path: 'frontend/admin/react/src/pages/app/notification/channel/index.tsx',
    expectStrings: [MASKS.notificationChannel],
    expectIdents: [],
    defs: [],
  },
  {
    // vben drawer form (inline literal) plus the vben-local inline enable
    // switch (narrow enabled-only update; see MASKS note).
    path: 'frontend/admin/vue-vben/apps/admin/src/views/app/notification/channel/index.vue',
    expectStrings: [
      MASKS.notificationChannel,
      MASKS.notificationChannelEnabledToggle,
    ],
    expectIdents: [],
    defs: [],
  },
  {
    // vue-element composable, inline literal.
    path: 'frontend/admin/vue-element/src/api/composables/notification-channel.ts',
    expectStrings: [MASKS.notificationChannel],
    expectIdents: [],
    defs: [],
  },

  // --- notification rule ---
  {
    // react edit form, inline literal.
    path: 'frontend/admin/react/src/pages/app/notification/rule/index.tsx',
    expectStrings: [MASKS.notificationRule],
    expectIdents: [],
    defs: [],
  },
  {
    // vben edit form, references the constant defined in the vben
    // composable (registered there).
    path: 'frontend/admin/vue-vben/apps/admin/src/views/app/notification/rule/index.vue',
    expectStrings: [],
    expectIdents: ['NOTIFICATION_RULE_UPDATE_MASK'],
    defs: [],
  },
  {
    // vue-element composable: defines the local constant and uses it in
    // the update call.
    path: 'frontend/admin/vue-element/src/api/composables/notification-rule.ts',
    expectStrings: [],
    expectIdents: ['UPDATE_MASK'],
    defs: [{ name: 'UPDATE_MASK', value: MASKS.notificationRule }],
  },
  {
    // vben composable: defines the constant referenced by the vben edit
    // form.
    path: 'frontend/admin/vue-vben/apps/admin/src/api/composables/notification-rule.ts',
    expectStrings: [],
    expectIdents: [],
    defs: [{ name: 'NOTIFICATION_RULE_UPDATE_MASK', value: MASKS.notificationRule }],
  },

  // --- notification template ---
  {
    // react edit form, inline literal.
    path: 'frontend/admin/react/src/pages/app/notification/template/index.tsx',
    expectStrings: [MASKS.notificationTemplate],
    expectIdents: [],
    defs: [],
  },
  {
    // vben edit form, references the constant defined in the vben
    // composable (registered there).
    path: 'frontend/admin/vue-vben/apps/admin/src/views/app/notification/template/index.vue',
    expectStrings: [],
    expectIdents: ['NOTIFICATION_TEMPLATE_UPDATE_MASK'],
    defs: [],
  },
  {
    // vue-element drawer, imports the constant aliased out of the
    // vue-element composable.
    path: 'frontend/admin/vue-element/src/pages/app/notification/template/notification-template-drawer.vue',
    expectStrings: [],
    expectIdents: ['NOTIFICATION_TEMPLATE_UPDATE_MASK'],
    defs: [],
  },
  {
    // vue-element composable: defines the constant (re-exported under the
    // NOTIFICATION_TEMPLATE_UPDATE_MASK alias). The alias re-export is
    // wiring — a broken import fails the vue-element typecheck — so only
    // the value is pinned here.
    path: 'frontend/admin/vue-element/src/api/composables/notification-template.ts',
    expectStrings: [],
    expectIdents: [],
    defs: [{ name: 'UPDATE_MASK', value: MASKS.notificationTemplate }],
  },
  {
    // vben composable: defines the constant referenced by the vben edit
    // form.
    path: 'frontend/admin/vue-vben/apps/admin/src/api/composables/notification-template.ts',
    expectStrings: [],
    expectIdents: [],
    defs: [{ name: 'NOTIFICATION_TEMPLATE_UPDATE_MASK', value: MASKS.notificationTemplate }],
  },

  // --- monitor alert ---
  {
    // react edit form, inline literal.
    path: 'frontend/admin/react/src/pages/app/system/monitor_alert/index.tsx',
    expectStrings: [MASKS.monitorAlert],
    expectIdents: [],
    defs: [],
  },
  {
    // vben edit form, inline literal.
    path: 'frontend/admin/vue-vben/apps/admin/src/views/app/system/monitor_alert/index.vue',
    expectStrings: [MASKS.monitorAlert],
    expectIdents: [],
    defs: [],
  },
  {
    // vue-element drawer, inline literal.
    path: 'frontend/admin/vue-element/src/pages/app/system/monitor_alert/monitor-alert-drawer.vue',
    expectStrings: [MASKS.monitorAlert],
    expectIdents: [],
    defs: [],
  },
];

// The three application source trees.
const ROOTS = [
  'frontend/admin/react/src',
  'frontend/admin/vue-element/src',
  'frontend/admin/vue-vben/apps/admin/src',
];

const SKIP_DIRS = new Set(['node_modules', 'dist', '.turbo', 'coverage']);
const SCANNABLE_RE = /\.(ts|tsx|vue|js|jsx|mjs)$/;

// updateMask usages. \s spans newlines, so newline-wrapped literals are
// caught too.
const STRING_USAGE_RE = /updateMask\s*:\s*(['"`])([^'"`\n]*)\1/g;
const IDENT_USAGE_RE = /updateMask\s*:\s*([A-Za-z_$][A-Za-z0-9_$]*)/g;
const UPDATEMASK_BINDING_RE = /\b(?:const|let|var)\s+updateMask\s*=/g;

function defPattern(name) {
  return new RegExp(
    "(?:export\\s+)?const\\s+" + name + "\\s*=\\s*(['\"`])([^'\"`\\n]*)\\1"
  );
}

function* walk(dir) {
  let entries;
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return; // unreadable directory (e.g. a root that no longer exists)
  }
  for (const entry of entries) {
    const child = `${dir}/${entry.name}`;
    if (entry.isDirectory()) {
      if (SKIP_DIRS.has(entry.name)) continue;
      yield* walk(child);
    } else if (SCANNABLE_RE.test(entry.name)) {
      yield child;
    }
  }
}

function tally(items) {
  const counts = new Map();
  for (const item of items) counts.set(item, (counts.get(item) ?? 0) + 1);
  return counts;
}

function diff(expected, found) {
  const expectedCounts = tally(expected);
  const foundCounts = tally(found);
  const missing = [];
  const extra = [];
  for (const [item, count] of expectedCounts) {
    if ((foundCounts.get(item) ?? 0) < count) missing.push(item);
  }
  for (const [item, count] of foundCounts) {
    if (count > (expectedCounts.get(item) ?? 0)) extra.push(item);
  }
  return { missing, extra };
}

const problems = [];
let scanned = 0;
const registryByPath = new Map(REGISTRY.map((entry) => [entry.path, entry]));
const visited = new Set();

for (const entry of REGISTRY) {
  if (!existsSync(entry.path)) {
    problems.push(
      `registered site file missing: ${entry.path} — the file moved or was deleted; update the registry in scripts/check-update-mask-allowlists.mjs`
    );
  }
}

for (const root of ROOTS) {
  for (const file of walk(root)) {
    if (file.includes('/api/generated/')) continue;
    scanned += 1;
    let content;
    try {
      content = readFileSync(file, 'utf8');
    } catch (err) {
      problems.push(`cannot read ${file}: ${err.message}`);
      continue;
    }
    const stringHits = [...content.matchAll(STRING_USAGE_RE)].map((m) => m[2]);
    const identHits = [...content.matchAll(IDENT_USAGE_RE)]
      .map((m) => m[1])
      .filter((name) => name !== 'makeUpdateMask');
    const bindingHits = content.match(UPDATEMASK_BINDING_RE) ?? [];
    const entry = registryByPath.get(file);
    if (!entry) {
      for (const value of stringHits) {
        problems.push(
          `unregistered updateMask literal in ${file}: ${JSON.stringify(value)} — derive masks with makeUpdateMask() from transport/rest, or consciously register the site in scripts/check-update-mask-allowlists.mjs`
        );
      }
      for (const name of identHits) {
        problems.push(
          `unregistered updateMask constant reference in ${file}: ${name} — register the site or switch to makeUpdateMask()`
        );
      }
      for (const hit of bindingHits) {
        problems.push(
          `unregistered updateMask binding in ${file} (${hit}) — bind the mask through makeUpdateMask(), not a local updateMask variable`
        );
      }
      continue;
    }
    visited.add(entry.path);
    const stringDiff = diff(entry.expectStrings, stringHits);
    for (const value of stringDiff.missing) {
      problems.push(
        `registered mask site missing in ${file}: expected ${JSON.stringify(value)} was not found — the site was removed or restructured; update the registry`
      );
    }
    for (const value of stringDiff.extra) {
      problems.push(
        `mask drift in ${file}: unexpected literal ${JSON.stringify(value)} — the allowlisted field set changed; reconcile across the three frontends, then update MASKS and the registry`
      );
    }
    const identDiff = diff(entry.expectIdents, identHits);
    for (const name of identDiff.missing) {
      problems.push(
        `registered constant reference missing in ${file}: expected ${name} was not found — the site was removed or restructured; update the registry`
      );
    }
    for (const name of identDiff.extra) {
      problems.push(
        `constant-reference drift in ${file}: unexpected reference ${name} — reconcile, then update the registry`
      );
    }
    for (const def of entry.defs) {
      const match = content.match(defPattern(def.name));
      if (!match) {
        problems.push(
          `mask constant ${def.name} not found in ${file} — the definition moved or was renamed; update the registry`
        );
        continue;
      }
      if (match[2] !== def.value) {
        problems.push(
          `mask constant ${def.name} in ${file} drifted: expected ${JSON.stringify(def.value)}, found ${JSON.stringify(match[2])} — reconcile across the three frontends, then update MASKS and the registry`
        );
      }
    }
  }
}

for (const entry of REGISTRY) {
  if (!visited.has(entry.path) && existsSync(entry.path)) {
    problems.push(
      `registered site file was not reached by the sweep: ${entry.path} — the ROOTS list no longer covers it`
    );
  }
}

if (problems.length > 0) {
  console.error(
    `update-mask allowlist check FAILED (${problems.length} problem(s)):`
  );
  for (const problem of problems) console.error('  - ' + problem);
  console.error(
    'fix: registered allowlist sites must carry exactly their registered\n' +
      'mask, and the allowlist is closed — new updateMask usage must go\n' +
      'through makeUpdateMask() from transport/rest or be consciously\n' +
      'registered in scripts/check-update-mask-allowlists.mjs.'
  );
  process.exit(1);
}

const pinned = REGISTRY.reduce(
  (sum, entry) =>
    sum + entry.expectStrings.length + entry.expectIdents.length + entry.defs.length,
  0
);

console.log(
  `update-mask allowlist check OK: ${scanned} file(s) scanned across ${ROOTS.length} frontend trees, ${pinned} registered allowlist assertion(s) verified, no unregistered updateMask usage.`
);
