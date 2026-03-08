/**
 * @cai/cli - Query and analyze AI coding history
 *
 * This package installs the CAI binary and provides a Node.js wrapper.
 * For full documentation, see: https://github.com/yourusername/coding-agent-insights
 */

module.exports = {
  name: '@cai/cli',
  version: require('./package.json').version,
  binary: require('./package.json').bin.cai
};
