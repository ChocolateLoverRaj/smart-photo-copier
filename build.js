const fsExtra = require('fs-extra')
const archiver = require('archiver')
const path = require('path')
const fs = require('fs')
const { once } = require('events')

const build = async () => {
    await fsExtra.ensureDir(path.join(__dirname, './dist/'))

    const archive = archiver('zip')
        .directory(path.join(__dirname, './smart-photo-copier-win32-x64/'), 'Smart Photo Copier')

    const writePromise = once(
        archive
            .pipe(fs.createWriteStream(path.join(__dirname, './dist/smart-photo-copier-win32-x64.zip'))),
        'close'
    )

    await archive.finalize()
    await writePromise
}

console.log('zipping...')
console.time('zip')
build()
    .then(() => {
        console.timeEnd('zip')
    })
    .catch(e => {
        console.error(e)
        process.exit(1)
    })
