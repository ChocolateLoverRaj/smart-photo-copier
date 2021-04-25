const { normalize: nodeNormalize } = require('path')

/**
 * This is more than just `path.normalize()`. 
 * This function simplifies any path so that it's equal to similar path.
 * 
 * For example: `c://A//b.txt\` is the same as `C:\\\\a\\\\B.TXT`.
 * In the example, both simplify to `c:\a\b.txt` (using forward / backwards slashes according to system).
 * 
 * @param path {string}
 */
const normalize = path => nodeNormalize(path)
    .replace(/[\/\\]$/, '')
    .toLowerCase()

module.exports = normalize
