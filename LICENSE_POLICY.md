# Public Repository License Policy

## Default

New public software repositories created by Three-Wheeled Sloth Studio should use **AGPL-3.0-only** unless a project-specific decision says otherwise.

This default is intended for applications and services where public inspection is useful and modified hosted versions should remain source-available to their users.

## Apply Deliberately

Do not stamp AGPL-3.0-only onto an existing repository without checking:

- Whether the repository already contains third-party code under incompatible terms
- Whether prior contributors own copyright in material that was accepted without a clear license agreement
- Whether the project is intended primarily as a reusable library where ecosystem compatibility is more important than network copyleft
- Whether the repository contains mostly documentation, data, game assets, fonts, music, or artwork that should use a content-specific license
- Whether an existing contract, client agreement, employer agreement, or distribution platform limits relicensing
- Whether the project must combine into a larger work whose license is incompatible with AGPL-3.0-only

## Existing Unlicensed Public Repositories

An unlicensed public repository is not automatically open source. Before adding AGPL-3.0-only:

1. Confirm that Three-Wheeled Sloth Studio controls the copyright for the material being licensed.
2. Inventory bundled dependencies, copied snippets, generated assets, and externally sourced content.
3. Separate code from content that needs a different license.
4. Add the full license text as `LICENSE`.
5. Add `AGPL-3.0-only` to package metadata where supported.
6. Add a short license statement to the README.
7. Record the decision in the repository's durable project references.

## Exceptions

Any exception should be explicit and written down. Common alternatives may include:

- Apache-2.0 or MIT for broadly reusable libraries and interoperability layers
- GPL-3.0-only for distributed software where the network-use provision is unnecessary
- A Creative Commons license for eligible documentation or art
- A proprietary or source-available license when open-source redistribution is not the goal

Licensing is a design decision, not a decorative sticker applied after the code has escaped into the yard.
