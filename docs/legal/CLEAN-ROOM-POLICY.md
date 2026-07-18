# FreeFlow Clean-Room Policy

Status: Binding project policy
Effective: 2026-07-17

## Purpose

FreeFlow is an independent implementation of generally useful desktop dictation workflows. This policy prevents proprietary code, confidential information, restricted service behavior, branding, and assets from entering the project.

This is an engineering control, not legal advice. Questions about statutory interoperability exceptions, trademarks, patents, or a proposed investigation outside this policy require qualified legal review before work begins.

## Permitted inputs

- Publicly accessible product documentation and marketing pages, used only to identify user-facing facts and workflows.
- Public user forums, reviews, and case-study accounts, used only as unverified workflow and risk hypotheses. Do not copy proprietary outputs, screenshots, or distinctive expression from them.
- Public operating-system documentation and accessibility APIs.
- Original requirements, test cases, recordings, prompts, copy, icons, and designs authored for FreeFlow.
- Open-source components whose licenses have been recorded and found compatible with FreeFlow distribution.
- Empirical testing of FreeFlow itself against independently authored corpora and task-success criteria.

## Prohibited inputs and methods

- Disassembling, decompiling, decrypting, or otherwise inspecting proprietary Wispr binaries or bundled resources.
- Network interception, undocumented endpoint discovery, certificate bypass, authentication bypass, scraping, bulk account creation, or access to non-public service areas.
- Extracting or copying proprietary source code, algorithms, prompts, model weights, UI assets, sounds, copy, icons, screenshots, or trade dress.
- Using Wispr inputs or outputs to train, distill, fine-tune, evaluate, or improve FreeFlow models.
- Circumventing subscriptions, usage caps, license checks, security controls, or technical protection measures.
- Claiming affiliation, endorsement, exact identity, or compatibility certification by Wispr.
- Recording or transcribing another person without the consent required by applicable law.

## Reference-product access

No FreeFlow prompt requires installing or operating Wispr Flow. Public documentation is the default reference boundary.

Any future proposal to conduct comparative testing against the reference service must be added as a reviewed PSPR addendum before execution. The addendum must identify the lawful basis, applicable terms, allowed observations, data-retention rules, and separation between observers and implementers. Without that addendum, such testing is out of scope.

## Contribution provenance

Every pull request that changes product behavior must identify:

1. the original requirement or public source that motivated it;
2. whether code was original, reused, extracted from an approved open-source dependency, or extended at an existing seam;
3. the dependency license and preserved notices where applicable; and
4. the verification evidence.

Contributors must not submit code or material learned from confidential employment, leaked source, proprietary reverse engineering, or other restricted access.

## Branding

- “FreeFlow” is a working name until a release-name and trademark clearance gate passes.
- Do not use Wispr’s name in the executable, package identifier, icon, screenshots, store copy, or primary product marketing.
- FreeFlow must use original visual design, wording, audio cues, icons, and motion.
- Factual comparison pages, if later approved, must be accurate, sourced, and clearly state non-affiliation.

## Enforcement

Suspect material is quarantined from the main branch while provenance is reviewed. If provenance cannot be established, the material is removed and reimplemented from the clean requirements by someone who has not inspected the suspect material.
