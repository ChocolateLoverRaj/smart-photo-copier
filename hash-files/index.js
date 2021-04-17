const { createHash } = require('crypto')
const envPaths = require('env-paths')
const makeDir = require('make-dir')
const { join } = require('path')
const fs = require('fs')
const events = require('events')

const paths = envPaths('hash-files')
const cachePath = paths.cache

let makeCacheDirPromise
const makeCacheDir = async () => {
  if (!makeCacheDirPromise) makeCacheDirPromise = makeDir(cachePath)
  return await makeCacheDirPromise
}

const getHash = async path => {
  // Name of folder is based off of path
  const hashName = createHash('sha1').update(path).digest('hex')
  const hashDirPath = join(cachePath, hashName)
  const hashFilePath = join(hashDirPath, 'hash.sha1')
  const sourceFilePath = join(hashDirPath, 'source.txt')

  // Check if path already exists
  let hashFileLastModified
  try {
    // Get the last modified of hash
    hashFileLastModified = (await fs.promises.stat(hashFilePath)).mtimeMs
  } catch (e) {
    // ENOENT is okay
    if (!e.code === 'ENOENT') throw e
  }

  let shouldUpdate = true
  if (hashFileLastModified) {
    // Check if the file was modified after hashing it
    const { mtimeMs } = await fs.promises.stat(path)
    if (mtimeMs < hashFileLastModified) shouldUpdate = false
  }

  if (shouldUpdate) {
    // Update the hash
    await makeCacheDir()
    await fs.promises.mkdir(hashDirPath, { recursive: true })
    await Promise.all([
      // Update the hash file
      events.once(
        fs.createReadStream(path)
          .pipe(createHash('sha1'))
          .pipe(fs.createWriteStream(hashFilePath)),
        'close'
      ),
      fs.promises.access(sourceFilePath)
        .catch(e => {
          if (e.code === 'ENOENT') return fs.promises.writeFile(sourceFilePath, path)
          else throw e
        })
    ])
  }
  // TODO: No need to read again from file if we just wrote to it
  // Read the hash file
  return await fs.promises.readFile(hashFilePath)
}

// TODO: Also make a function that deletes hashes of non-existent paths
module.exports = getHash
