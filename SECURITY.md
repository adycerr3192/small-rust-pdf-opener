# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Email or message the maintainer via GitHub (`will702`) with:

- Description of the issue
- Steps to reproduce
- Impact assessment (if known)
- Whether you plan to disclose publicly and on what timeline

We aim to acknowledge reports within a reasonable time and coordinate fixes before public disclosure when appropriate.

## Scope notes

- This app processes local PDFs; treat untrusted PDFs carefully (PDF parsers can be attack surfaces).
- Certificate signing uses OpenSSL; keep your PKCS#12 files private.
- OCR model downloads use HTTPS; verify network policy in locked-down environments.
