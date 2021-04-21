const { dialog } = require('electron').remote
const { list: recursiveReaddir } = require('recursive-readdir-async')
const fsExtra = require('fs-extra')
const fs = require('fs')
const { join, basename, dirname, extname, relative } = require('path')
const getHash = require('hash-files')

const form = document.getElementById('form')
const existSpan = document.getElementById('form__exist-span')
const srcSpan = document.getElementById('form__src-span')
const destSpan = document.getElementById('form__dest-span')
const progressDiv = document.getElementById('progress')
const gridCopied = document.getElementById('grid__copied')
const gridDuplicate = document.getElementById('grid__duplicate')
const gridRemaining = document.getElementById('grid__remaining')
const textareaCopied = document.getElementById('grid__copied-list')
const textareaDuplicate = document.getElementById('grid__duplicate-list')
const textareaRemaining = document.getElementById('grid__remaining-list')
const stepsList = document.getElementById('steps')

form.addEventListener('submit', e => {
    e.preventDefault()
})

const dirs = {
    exist: {
        button: form.existButton,
        input: form.exist,
        span: existSpan,
        valid: false
    },
    src: {
        button: form.srcButton,
        input: form.src,
        span: srcSpan,
        valid: false
    },
    dest: {
        button: form.destButton,
        input: form.dest,
        span: destSpan,
        valid: false
    }
}

const updateCopyButton = () => {
    let disabled = false
    for (const { valid } of Object.values(dirs)) {
        if (!valid) {
            disabled = true
            break
        }
    }
    form.copy.disabled = disabled
}

Object.values(dirs).forEach(dir => {
    const checkDirectory = () => {
        const path = dir.input.value.trim()
        dir.valid = false
        if (path) {
            if (fs.existsSync(path)) {
                dir.span.innerText = 'Folder Exists'
                dir.valid = true
            } else {
                dir.span.innerText = 'Folder doesn\'t exist'
            }
        } else {
            dir.span.innerText = 'Not Selected'
        }
        updateCopyButton()
    }
    dir.button.addEventListener('click', () => {
        const path = dialog.showOpenDialogSync({ properties: ['openDirectory'] })
        if (path) {
            dir.input.value = path[0]
            checkDirectory()
        }
    })
    dir.input.addEventListener('input', checkDirectory)
})

let currentStep = 0
const filesCopied = new Set()
const duplicateFiles = new Set()
const remainingFiles = new Set()
const updateProgress = () => {
    for (let i = 0; i < stepsList.children.length; i++) {
        const elem = stepsList.children[i]
        elem.classList.remove('in-progress')
        elem.classList.remove('complete')
        if (i < currentStep) {
            elem.classList.add('complete')
        } else if (i === currentStep) {
            elem.classList.add('in-progress')
        }
    }
    gridCopied.innerText = filesCopied.size
    gridDuplicate.innerText = duplicateFiles.size
    gridRemaining.innerText = remainingFiles.size
    textareaCopied.innerHTML = [...filesCopied].join('\n')
    textareaDuplicate.innerHTML = [...duplicateFiles].join('\n')
    textareaRemaining.innerHTML = [...remainingFiles].join('\n')
}

const disable = disable => {
    // Disable copy button
    form.copy.disabled = disable
    // Disable everything
    for (const { button, input } of Object.values(dirs)) {
        button.disabled = disable
        input.disabled = disable
    }
}

const exists = async path => {
    try {
        await fs.promises.access(path)
        return true
    } catch (e) {
        if (e.code !== 'ENOENT') throw e
    }
    return false
}

form.copy.addEventListener('click', async () => {
    // Disable everything
    disable(true)
    // Show progress
    progressDiv.classList.remove('hidden')
    // Reset progress
    currentStep = 0
    filesCopied.clear()
    duplicateFiles.clear()
    remainingFiles.clear()
    updateProgress()
    // Read the exist dir
    const [readExist, readSrc] = await Promise.all([
        recursiveReaddir(form.exist.value),
        recursiveReaddir(form.src.value)
    ])
    currentStep++
    updateProgress()
    // Copy time
    const createDirs = new Map()
    const createDir = path => {
        if (path === '.') {
            return
        }
        if (createDirs.has(path)) {
            return createDirs.get(path)
        }
        const promise = (async () => {
            const parts = path.split('/')
            if (parts.length > 1) {
                await createDir([].concat(...parts.slice(0, parts.length - 1)))
            }
            await fsExtra.ensureDir(join(form.dest.value, path))
        })()
        createDirs.set(path, promise)
        return promise
    }
    await Promise.all(readSrc.map(async ({ fullname: srcFilePath }) => {
        const relativePath = relative(form.src.value, srcFilePath)
        const srcFileHash = await getHash(srcFilePath)
        const duplicateOf = (await Promise.all(readExist.map(async ({ fullname: existFilePath }) => (
            srcFileHash.compare(await getHash(existFilePath)) === 0
        )))).indexOf(true)
        if (duplicateOf !== -1) {
            const relativeDuplicatePath = relative(form.dest.value, readExist[duplicateOf].fullname)
            duplicateFiles.add(`${relativePath} - duplicate of ${relativeDuplicatePath}`)
            updateProgress()
            return
        }
        remainingFiles.add(relativePath)
        updateProgress()
        const destExt = extname(srcFilePath)
        const destBasename = basename(srcFilePath)
        const relativePathDirname = dirname(relativePath)
        let destBasenameToUse = destBasename
        let destPathToUse
        for (let i = 2; await exists(destPathToUse = join(form.dest.value, relativePathDirname, destBasenameToUse)); i++) {
            destBasenameToUse = `${destBasename} (${i})${destExt}`
        }
        await createDir(relativePathDirname)
        await fs.promises.copyFile(srcFilePath, destPathToUse)
        remainingFiles.delete(relativePath)
        const copiedFileShown = relativePath + (destBasenameToUse !== relativePath
            ? ` --> ${destBasenameToUse}`
            : ''
        )
        filesCopied.add(copiedFileShown)
        updateProgress()
    }))
    currentStep++
    updateProgress()
    disable(false)
})
