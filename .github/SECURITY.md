# Security Policy

## Supported Versions

Currently, only the latest released version of CAI is supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |

## Reporting a Vulnerability

The CAI team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

If you discover a security vulnerability, please report it as follows:

### Private Disclosure

1. **Do not create a public issue** for security vulnerabilities
2. Send an email to: [INSERT SECURITY EMAIL]
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if known)

### What to Expect

- **Initial Response**: We will acknowledge receipt within 48 hours
- **Timeline**: We aim to resolve security issues within 7-14 days
- **Updates**: We'll keep you informed of our progress
- **Disclosure**: We will coordinate public disclosure with you

### Public Disclosure Process

1. After validation, we will create a security advisory
2. We will develop and test a fix
3. We will release a new version with the fix
4. We will publish the security advisory
5. We will credit you for the discovery (with your permission)

## Security Best Practices

### For Users

- **Keep CAI Updated**: Always use the latest version
- **Verify Downloads**: Only download from official sources
- **Check Permissions**: Review what CAI accesses
- **Report Suspicious Activity**: If you notice unexpected behavior

### For Developers

- **Input Validation**: Always validate user input
- **Dependencies**: Keep dependencies updated
- **Secrets Management**: Never commit secrets or API keys
- **Code Review**: All code goes through review process

## Dependency Security

CAI uses automated security scanning:

- **GitHub Dependabot**: Automated dependency updates
- **Cargo Audit**: Security audit of dependencies
- **CI/CD Checks**: Security checks run on every PR

### Vulnerability Response

When a dependency vulnerability is discovered:

1. **Immediate Assessment**: Determine impact on CAI
2. **Update**: Update to patched version if available
3. **Workaround**: Document temporary mitigations if needed
4. **Release**: Release security update as soon as possible

## Security Features

### Current Implementation

- **Input Sanitization**: All user input is validated
- **SQL Injection**: Parameterized queries prevent SQL injection
- **Path Traversal**: File access is restricted to safe paths
- **Resource Limits**: Built-in limits prevent resource exhaustion

### Planned Enhancements

- [ ] Signature verification for releases
- [ ] Credential management for remote sources
- [ ] Audit logging for sensitive operations
- [ ] Sandboxing for plugin execution

## Security Audits

The CAI project welcomes professional security audits.

If you're interested in sponsoring a security audit, please contact:
[INSERT CONTACT EMAIL]

## Related Resources

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Reporting Issues](https://github.com/cai-dev/coding-agent-insights/issues)

## Receiving Security Notifications

To receive security notifications:

1. Watch the CAI repository on GitHub
2. Enable "Customize" → "Security alerts"
3. Monitor [Security Advisories](https://github.com/cai-dev/coding-agent-insights/security/advisories)

## Security Hall of Fame

We credit security researchers who help make CAI more secure:

- [ ] Your name here!

Thank you for helping keep CAI secure!
