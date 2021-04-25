const { createHash } = require('crypto')
const envPaths = require('env-paths')
const makeDir = require('make-dir')
const { join } = require('path')
const fs = require('fs')
const events = require('events')
const throttleAsyncFn = require('throttle-async-function')
const normalize = require('./normalize')

const paths = envPaths('hash-files')
const cachePath = paths.cache

let makeCacheDirPromise
const makeCacheDir = async () => {
  if (!makeCacheDirPromise) makeCacheDirPromise = makeDir(cachePath)
  return await makeCacheDirPromise
}

const getHash = throttleAsyncFn(async path => {
  // Name of folder is based off of path
  const normalizedPath = normalize(path)
  const hashName = createHash('sha1').update(normalizedPath).digest('hex')
  const hashDirPath = join(cachePath, hashName)
  const hashFilePath = join(hashDirPath, 'hash.sha1')
  const sourceFilePath = join(hashDirPath, 'source.txt')

  // Check if path already exists
  let hashFileLastModified
  try {
    console.log('Using fs', path)
    // Get the last modified of hash
    hashFileLastModified = (await fs.promises.stat(hashFilePath)).mtimeMs
  } catch (e) {
    // ENOENT is okay
    if (!e.code === 'ENOENT') throw e
  }

  let shouldUpdate = true
  if (hashFileLastModified) {
    // Check if the file was modified after hashing it
    const { mtimeMs } = await fs.promises.stat(normalizedPath)
    if (mtimeMs < hashFileLastModified) shouldUpdate = false
  }

  if (shouldUpdate) {
    // Update the hash
    await makeCacheDir()
    await fs.promises.mkdir(hashDirPath, { recursive: true })
    await Promise.all([
      // Update the hash file
      events.once(
        fs.createReadStream(normalizedPath)
          .pipe(createHash('sha1'))
          .pipe(fs.createWriteStream(hashFilePath)),
        'close'
      ),
      fs.promises.access(sourceFilePath)
        .catch(e => {
          if (e.code === 'ENOENT') return fs.promises.writeFile(sourceFilePath, normalizedPath)
          else throw e
        })
    ])
  }
  // TODO: No need to read again from file if we just wrote to it
  // Read the hash file
  return await fs.promises.readFile(hashFilePath)
}, {
  // Only check hash once after process is started
  cacheRefreshPeriod: Infinity
})

// TODO: Also make a function that copies the hash of a file to another file

// TODO: Also make a function that deletes hashes of non-existent paths
module.exports = getHash
